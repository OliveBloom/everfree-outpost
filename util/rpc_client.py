"""Client script for a more user-friendly Everfree Outpost Python REPL.

Usage:
    OUTPOST_REPL_SOCKET=.../path/to/outpost/repl  python3 -i rpc_client.py

Example:
    >>> eng.num_clients()
    1
    >>> client = eng.client_by_name('OP')
    <[remote] client #0>
    >>> client.pawn().pos()
    V3(32, 32, 0)
    >>> client.pawn().plane().name
    'Everfree Forest'

The script provides access to the sever through several mechanisms:
 * `eng`: the server-side `EngineProxy`
 * `ctx`: the server-side `RPCContext` (in outpost_server.core.eval).  See
   below for info on the RPC system.
 * `outpost_server` package: You can directly import any module from this
   package.

This script implements a simple RPC mechanism for interacting with objects in
the remote interpreter (the one embedded in the Outpost server).  For example,
the `eng` object provided by the script is actually an `RPCObject` that refers
to an `EngineProxy` object on the server.  (This is visible from the "[remote]"
tag in its `repr` and by examining `type(eng)`.)  However, `eng` behaves (for
the most part) as if it were actually the remote `EngineProxy` object: it has
the same attributes, methods, etc. as the real `EngineProxy`, which can all be
accessed using the normal Python syntax.

In addition to the `outpost_server` package mentioned above, this script also
provides a `remote` package.  Running `from remote.foo import bar` will import
the `foo.bar` module on the server and set `bar` to be an `RPCObject` referring
to that module.  In particular, this may be useful for accessing server-side
builtins, through `import remote.builtins`.
"""

import ast
import builtins
import io
import pickle
import socket
import sys

class RPCError(Exception):
    pass

class RPCClient(object):
    def __init__(self, path):
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

    def import_raw(self, name, fromlist=()):
        return self.operate('import_raw', (name, fromlist))

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

    def install_importer(self):
        importer = RemoteImporter(self)
        importer.register('remote', '')
        importer.register('outpost_server', 'outpost_server')
        importer.install()

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

remote_prop = lambda name: \
        property(lambda self: rawget(self, '_client').getattr(self, name))

class RPCObject(object):
    def __init__(self, client, pid):
        object.__setattr__(self, '_client', client)
        object.__setattr__(self, '_pid', pid)

    def __repr__(self):
        result = rawget(self, '_client').call_method(self, '__repr__')
        if result.startswith('<') and result.endswith('>'):
            result = '<[remote] ' + result[1:]
        return result

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

    __str__ = remote_prop('__str__')
    __len__ = remote_prop('__len__')
    __iter__ = remote_prop('__iter__')
    __next__ = remote_prop('__next__')
    __getitem__ = remote_prop('__getitem__')
    __setitem__ = remote_prop('__setitem__')
    __delitem__ = remote_prop('__delitem__')

class CallableRPCObject(RPCObject):
    def __call__(self, *args, **kwargs):
        return rawget(self, '_client').call(self, *args, **kwargs)


class FakePackage(object):
    """Object that emulates a package with an empty `__init__.py`.  Useful for
    virtual packages that don't have an `__init__.py` anywhere on disk."""
    def __init__(self, name):
        self.__name__ = name
        self.__package__ = name
        self.__path__ = ()
        self.__all__ = ()

class RemoteImport(object):
    def __init__(self, client):
        self._orig_import = None
        self._client = client
        self._name_map = {}

    def install(self):
        assert not self.installed()
        self._orig_import = builtins.__import__
        builtins.__import__ = self

        for local, remote in self._name_map.items():
            self._install_module(local, remote)

    def installed(self):
        return self._orig_import is not None

    def _install_module(self, local, remote):
        local_name = local[:-1]
        if remote == '':
            sys.modules[local_name] = FakePackage(local_name)
        else:
            remote_name = remote[:-1]
            sys.modules[local_name] = self._client.import_raw(remote_name)

    def register(self, local_prefix, remote_prefix):
        local = local_prefix + '.'
        remote = remote_prefix + '.' if remote_prefix is not None else ''
        self._name_map[local] = remote

        if self.installed():
            self._install_module(local, remote)

    def __call__(self, name, globals=None, locals=None, fromlist=(), level=0):
        for local, remote in self._name_map.items():
            if name.startswith(local):
                remote_name = remote + name[len(local):]
                if level != 0:
                    raise ImportError("can't use relative imports for remote packages")

                result = self._client.import_raw(remote_name, fromlist)
                if remote == '' and not fromlist:
                    basename, _, _ = remote_name.partition('.')
                    parent = sys.modules[local[:-1]]
                    setattr(parent, basename, result)
                    return parent
                else:
                    return result
        return self._orig_import(name, globals, locals, fromlist, level)

if __name__ == '__main__':
    import os
    C = RPCClient(os.environ.get('OUTPOST_REPL_SOCKET', 'repl'))
    ctx = C.ctx
    eng = C.ctx.engine

    IMPORTER = RemoteImport(C)
    IMPORTER.register('remote', None)
    IMPORTER.register('outpost_server', 'outpost_server')
    IMPORTER.install()
