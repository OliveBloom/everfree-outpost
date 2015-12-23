import ast
import builtins
import io
import os
import pickle
import socket

class RPCError(Exception):
    pass

class RPCClient(object):
    def __init__(self, path=None):
        path = path or os.environ.get('OUTPOST_REPL_SOCKET', 'repl')

        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(path)

        self._conn = sock.makefile(mode='rw')

    def operate(self, func_name, args):
        args_rep = self.pickle(args)
        code = '@RPC_CONTEXT.operate(%r, %r)\n' % (func_name, args_rep)
        self._conn.write(code)
        self._conn.flush()

        result_code = self._conn.readline()
        result_rep = ast.literal_eval(result_code)
        status, result = self.unpickle(result_rep)

        if status == 'ok':
            return result
        elif status == 'exc':
            ty_name, val, tb = result
            ty = getattr(builtins, ty_name, None)
            if ty is None:
                ty = RuntimeError
                val = '[%s] %s' % (ty_name, val)
            raise ty(val) from RPCError(tb.strip())
        else:
            assert False, 'unrecognized status code: %r' % status

    @property
    def ctx(self):
        return self.operate('context', ())

    def import_(self, name):
        return self.operate('import', (name,))

    def getattr(self, obj, key):
        return self.operate('getattr', (obj, key))

    def call(self, obj, *args, **kwargs):
        return self.operate('call', (obj, args, kwargs))

    def call_method(self, obj, method, *args, **kwargs):
        return self.operate('call_method', (obj, method, args, kwargs))

    def release(self, pid):
        try:
            self._conn.write('@RPC_CONTEXT.release_obj(%r)\n' % int(pid))
            self._conn.flush()
            _ = self._conn.readline()
            # Discard result.  We can't really clean up if anything happens.
        except Exception as e:
            pass

    def pickle(self, obj):
        buf = io.BytesIO()
        pickler = RPCClientPickler(self, buf, protocol=3)
        pickler.dump(obj)
        return buf.getvalue()

    def unpickle(self, rep):
        buf = io.BytesIO(rep)
        unpickler = RPCClientUnpickler(self, buf)
        return unpickler.load()

rawget = object.__getattribute__

class RPCClientPickler(pickle.Pickler):
    def __init__(self, client, *args, **kwargs):
        super(RPCClientPickler, self).__init__(*args, **kwargs)
        self._client = client

    def persistent_id(self, obj):
        if isinstance(obj, RPCObject):
            assert rawget(obj, '_client') is self._client, \
                    'tried to send RPCObject that belongs to a different context'
            return rawget(obj, '_pid')
        elif obj is None or type(obj) in \
                (bool, int, float, complex, tuple, range, str, bytes, frozenset,
                    list, bytearray, set, dict):
            return None
        else:
            assert False, "can't send %r over rpc" % type(obj)

class RPCClientUnpickler(pickle.Unpickler):
    def __init__(self, client, *args, **kwargs):
        super(RPCClientUnpickler, self).__init__(*args, **kwargs)
        self._client = client

    def persistent_load(self, pid):
        # This separation is pretty gross, but it's the only way to make the
        # built-in `callable` function behave correctly.
        if pid.startswith('c'):
            return CallableRPCObject(self._client, pid[1:])
        else:
            return RPCObject(self._client, pid)

class RPCObject(object):
    def __init__(self, client, pid):
        object.__setattr__(self, '_client', client)
        object.__setattr__(self, '_pid', pid)

    def __repr__(self):
        return rawget(self, '_client').call_method(self, '__repr__')

    def __str__(self):
        return rawget(self, '_client').call_method(self, '__str__')

    def __dir__(self):
        return rawget(self, '_client').operate('dir', (self,))

    def __getattribute__(self, key):
        if key == '__class__':
            # For some reason, tab completion breaks if __class__ doesn't give
            # valid results.
            return rawget(self, key)
        return rawget(self, '_client').getattr(self, key)

    def __del__(self):
        rawget(self, '_client').release(rawget(self, '_pid'))

class CallableRPCObject(RPCObject):
    def __call__(self, *args, **kwargs):
        return rawget(self, '_client').call(self, *args, **kwargs)
