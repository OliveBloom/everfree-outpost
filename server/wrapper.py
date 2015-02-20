import binascii
import os
import socket
import struct
import time

import tornado.autoreload
import tornado.gen
import tornado.ioloop
import tornado.iostream
import tornado.netutil
import tornado.process
import tornado.web
import tornado.websocket


OP_ADD_CLIENT =     0xff00
OP_REMOVE_CLIENT =  0xff01
OP_CLIENT_REMOVED = 0xff02
OP_REPL_COMMAND =   0xff03
OP_REPL_RESULT =    0xff04


def now():
    t = time.time()
    return int(t * 1000)

LOG_DIR = None


class WSHandler(tornado.websocket.WebSocketHandler):
    def __init__(self, *args, **kwargs):
        super(WSHandler, self).__init__(*args, **kwargs)
        self.log = None
        self.alive = True

    def open(self):
        self.id = backend.add_client(self)
        if LOG_DIR is not None:
            # IPv6 addresses can be 4-tuples instead of pairs
            host, port = self.request.connection.address[:2]
            filename = '%d-%s-%d.log' % (now(), host, port)
            self.log = open(os.path.join(LOG_DIR, filename), 'w')

    def on_message(self, message):
        if not self.alive:
            return
        if self.log is not None:
            self.log.write('%d: > %s\n' % (now(), binascii.hexlify(message)))
        backend.send_client_message(self.id, message)

    def on_close(self):
        if not self.alive:
            return
        if self.log is not None:
            self.log.write('%d: close\n' % now())
        backend.remove_client(self.id)
        self.alive = False

    def write_message(self, msg, *args, **kwargs):
        self.log.write('%d: < %s\n' % (now(), binascii.hexlify(msg)))
        super(WSHandler, self).write_message(msg, *args, **kwargs)

    def close(self):
        super(WSHandler, self).close()
        # Make extra sure we stop processing messages once the socket is
        # closed.  Otherwise we might close the socket, reuse the ID, and then
        # interpret messages from the old connection as if they came from the
        # new one.
        self.alive = False

class FileHandler(tornado.web.StaticFileHandler):
    @classmethod
    def get_absolute_path(cls, root, path):
        if path == '':
            return super(FileHandler, cls).get_absolute_path(root, 'client.html')
        return super(FileHandler, cls).get_absolute_path(root, path)

application = tornado.web.Application([
    (r'/ws', WSHandler),
    (r'/(.*)', FileHandler, { 'path': './dist/www' }),
], debug=True)


class BackendStream(object):
    def __init__(self, args, io_loop=None):
        self.io_loop = io_loop or tornado.ioloop.IOLoop.current()
        self.impl = tornado.process.Subprocess(
                args,
                stdout=tornado.process.Subprocess.STREAM,
                stdin=tornado.process.Subprocess.STREAM,
                io_loop=self.io_loop)

        self.clients = { 0: None }
        self.unused_ids = set()
        self.closed_ids = set()

        self.io_loop.add_future(self.do_read(), lambda f: print('read done?', f.result()))
        self.impl.set_exit_callback(self.on_close)

        self.repl_result_callback = lambda c, m: None

    def add_client(self, conn):
        if len(self.unused_ids) > 0:
            id = self.unused_ids.pop()
        else:
            id = len(self.clients)

        assert id < 1 << 16

        self.clients[id] = conn
        self.send_special_client_message(id, OP_ADD_CLIENT)
        return id

    def remove_client(self, id):
        self.closed_ids.add(id)
        self.send_special_client_message(id, OP_REMOVE_CLIENT)

    def send_client_message(self, id, msg):
        if len(msg) < 2 or len(msg) >= 1 << 16:
            self.remove_client(id)
            return

        if isinstance(msg, str):
            msg = msg.encode()

        header = struct.pack('HH', id, len(msg))
        self.impl.stdin.write(header)
        self.impl.stdin.write(msg)

    def send_special_client_message(self, id, opcode):
        self.impl.stdin.write(struct.pack('HHHH', 0, 4, opcode, id))

    def send_repl_command(self, cookie, body):
        header = struct.pack('HHHH', 0, len(body) + 4, OP_REPL_COMMAND, cookie)
        self.impl.stdin.write(header + body)

    @tornado.gen.coroutine
    def do_read(self):
        while True:
            header = yield tornado.gen.Task(self.impl.stdout.read_bytes, 4)
            id, size = struct.unpack('HH', header)
            body = yield tornado.gen.Task(self.impl.stdout.read_bytes, size)

            if id == 0:
                opcode, = struct.unpack_from('H', body, 0)

                if opcode == OP_CLIENT_REMOVED:
                    id, = struct.unpack_from('H', body, 2)
                    if id in self.clients:
                        # close() is a no-op if it's already been closed
                        self.clients[id].close()
                        del self.clients[id]
                    if id in self.closed_ids:
                        self.closed_ids.remove(id)
                    self.unused_ids.add(id)
                elif opcode == OP_REPL_RESULT:
                    cookie, = struct.unpack_from('H', body, 2)
                    self.repl_result_callback(cookie, body[4:])
                else:
                    assert False, \
                            'bad opcode in control response'
                continue

            # Only dispatch the message if the target is still connected.
            if id not in self.closed_ids:
                self.clients[id].write_message(body, binary=True)

    def on_close(self, code):
        print('closed!')


class ReplServer(object):
    def __init__(self, backend, io_loop=None):
        self.io_loop = io_loop or tornado.ioloop.IOLoop.current()
        if hasattr(tornado.netutil, 'bind_unix_socket'):
            self.server_sockets = [tornado.netutil.bind_unix_socket('./repl')]
        else:
            self.server_sockets = tornado.netutil.bind_sockets(9999, address='localhost')

        for sock in self.server_sockets:
            tornado.netutil.add_accept_handler(sock, self.on_connect)

        self.pending = {}
        self.backend = backend

        backend.repl_result_callback = self.on_reply

    def on_connect(self, sock, addr):
        conn = tornado.iostream.IOStream(sock)
        self.io_loop.add_future(self.do_read(conn), lambda f: None)
        conn.set_close_callback(lambda: self.on_close(conn))

    def on_close(self, conn):
        pending_ids = set()
        for k,v in self.pending.items():
            if v[0] is conn:
                pending_ids.add(k)
        for k in pending_ids:
            del self.pending[k]

    def next_cookie(self, conn, cb):
        for i in range(0, len(self.pending) + 1):
            if i not in self.pending:
                self.pending[i] = (conn, cb)
                return i
        assert False, 'unreachable'

    def send_repl_command(self, conn, text, callback):
        cookie = self.next_cookie(conn, callback)
        self.backend.send_repl_command(cookie, text)

    @tornado.gen.coroutine
    def do_read(self, conn):
        def read_line():
            return tornado.gen.Task(conn.read_until, b'\n')

        while not conn.closed():
            line = yield read_line()
            if line.strip() != b'{':
                text = line
            else:
                lines = []
                while True:
                    line = yield read_line()
                    if line.strip() == b'}':
                        break
                    lines.append(line)
                text = b''.join(lines)

            result = yield tornado.gen.Task(self.send_repl_command, conn, text)
            conn.write(result)

    def on_reply(self, cookie, text):
        if cookie not in self.pending:
            print('unexpected reply cookie: %d' % cookie)
            return

        _, cb = self.pending.pop(cookie)
        cb(text)

if __name__ == "__main__":
    import sys

    bin_dir = os.path.dirname(sys.argv[0])
    root_dir = os.path.dirname(bin_dir)

    LOG_DIR = os.path.join(root_dir, 'logs')
    os.makedirs(LOG_DIR, exist_ok=True)

    exe = os.path.join(root_dir, 'bin/backend')
    blocks_json = os.path.join(root_dir, 'data/blocks.json')
    objects_json = os.path.join(root_dir, 'data/objects.json')
    script_dir = os.path.join(root_dir, 'scripts')

    tornado.autoreload.watch(exe)
    tornado.autoreload.watch(blocks_json)
    tornado.autoreload.watch(objects_json)
    tornado.autoreload.watch(os.path.join(script_dir, 'bootstrap.lua'))

    backend = BackendStream([exe, root_dir])
    repl = ReplServer(backend)
    application.listen(8888)
    tornado.ioloop.IOLoop.instance().start()
