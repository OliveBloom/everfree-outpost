import ast
import builtins
import importlib
import io
import pickle
import sys
import traceback

from outpost_server.core.engine import EngineProxy

class RPCContext(object):
    def __init__(self):
        self._eng = None
        self.engine = EngineProxy(RPCEngineRef(self))
        self._objs = []
        self._free_objs = []

    def retain_obj(self, obj):
        if len(self._free_objs) > 0:
            idx = self._free_objs.pop()
            self._objs[idx] = obj
        else:
            idx = len(self._objs)
            self._objs.append(obj)
        return idx

    def release_obj(self, idx):
        self._objs[idx] = None
        self._free_objs.append(idx)

    def get_obj(self, idx):
        return self._objs[idx]

    def reset(self):
        self._objs = []
        self._free_objs = []

    def set_engine_ref(self, eng):
        self._eng = eng

    def run(self, source, local_vals={}, local_objs={}):
        locals_ = local_vals.copy()
        for k,v in local_objs:
            locals_[k] = self.get_obj(v)

        globals_ = {
                'ctx': self,
                'eng': self.engine,
                '__builtins__': builtins,
                }

        # TODO

    def pickle(self, obj):
        buf = io.BytesIO()
        pickler = RPCServerPickler(self, buf, protocol=3)
        pickler.dump(obj)
        return buf.getvalue()

    def unpickle(self, rep):
        buf = io.BytesIO(rep)
        unpickler = RPCServerUnpickler(self, buf)
        return unpickler.load()

    def operate(self, func_name, args_rep):
        args = self.unpickle(args_rep)

        try:
            f = getattr(self, 'op_' + func_name)
            result = f(*args)
            result_rep = self.pickle(('ok', result))
        except Exception as e:
            ty, val, tb = sys.exc_info()
            tb_str = traceback.format_exc()
            result_rep = self.pickle(('exc', (ty.__name__, val, tb_str)))

        return repr(result_rep)

    def op_context(self):
        return self

    def op_import(self, name):
        return importlib.import_module(name)

    def op_import_raw(self, name, fromlist=()):
        return builtins.__import__(name, fromlist=fromlist)

    def op_getattr(self, obj, key):
        return getattr(obj, key)

    def op_dir(self, obj):
        return tuple(dir(obj))

    def op_call(self, obj, args, kwargs):
        return obj(*args, **kwargs)

    def op_call_method(self, obj, method, args, kwargs):
        return getattr(obj, method)(*args, **kwargs)

    def op_release(self, pid):
        self.release_obj(int(pid))


class RPCEngineRef(object):
    def __init__(self, ctx):
        self._ctx = ctx

    def __getattr__(self, key):
        return getattr(self._ctx._eng, key)

RPC_CONTEXT = RPCContext()

class RPCServerPickler(pickle.Pickler):
    def __init__(self, ctx, *args, **kwargs):
        super(RPCServerPickler, self).__init__(*args, **kwargs)
        self._ctx = ctx

    def persistent_id(self, obj):
        if obj is None or type(obj) in \
                (bool, int, float, complex, tuple, range, str, bytes, frozenset):
            return None
        else:
            pid = str(self._ctx.retain_obj(obj))
            if callable(obj):
                pid = 'c' + pid
            return pid

class RPCServerUnpickler(pickle.Unpickler):
    def __init__(self, ctx, *args, **kwargs):
        super(RPCServerUnpickler, self).__init__(*args, **kwargs)
        self._ctx = ctx

    def persistent_load(self, pid):
        return self._ctx.get_obj(int(pid))

def eval_or_exec(code_str, globals, locals):
    code = ast.parse(code_str)
    if len(code.body) == 1 and isinstance(code.body[0], ast.Expr):
        code = ast.Expression(code.body[0].value)
        code = compile(code, '<unknown>', 'eval')
        return eval(code, globals, locals)
    else:
        code = compile(code, '<unknown>', 'exec')
        exec(code, globals, locals)
        return None

def do_eval(eng, code):
    RPC_CONTEXT.set_engine_ref(eng)
    locals_ = {
            'RPC_CONTEXT': RPC_CONTEXT,
            'eng': EngineProxy(eng),
            }
    try:
        return str(eval_or_exec(code, {}, locals_))
    except Exception as e:
        return repr(e)

def init(hooks):
    hooks.eval(do_eval)
