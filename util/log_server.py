import os

import tornado.ioloop
import tornado.web
import tornado.websocket


CONNS = []

class WSHandler(tornado.websocket.WebSocketHandler):
    def __init__(self, *args, **kwargs):
        super(WSHandler, self).__init__(*args, **kwargs)

        idx = None
        for i in range(len(CONNS)):
            if CONNS[i] is None:
                idx = i
                break
        else:
            idx = len(CONNS)
            CONNS.append(None)

        CONNS[idx] = self
        self.conn_id = idx

    def check_origin(self, origin):
        return True

    def open(self):
        self._output(' == connected ==')

    def on_message(self, message):
        self._output(message)

    def on_close(self):
        self._output(' == disconnected ==')
        CONNS[self.conn_id] = None

    def _output(self, msg):
        print('%s%d:%s' % ('\t' * self.conn_id, self.conn_id, msg))

application = tornado.web.Application([
    (r'/log', WSHandler),
], debug=True)

if __name__ == "__main__":
    PORT = int(os.environ.get('OUTPOST_LOG_SERVER_PORT', 8892))
    application.listen(PORT)
    tornado.ioloop.IOLoop.instance().start()
