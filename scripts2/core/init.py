import builtins
import sys

from outpost_server import core
import outpost_server.core.data
import outpost_server.core.eval


old_print = builtins.print
def err_print(*args, file=sys.stderr, flush=True, **kwargs):
    old_print(*args, file=file, flush=flush, **kwargs)
builtins.print = err_print

def startup(eng):
    from outpost_server.core.data import DATA
    sys.stderr.write('hello %s\n' % eng)
    sys.stderr.write('hello %s\n' % DATA.recipe('anvil'))
    sys.stderr.write('hello %s\n' % eng.now())
    sys.stderr.flush()

def init(storage, data, hooks):
    core.data.init(data)
    core.eval.init(hooks)

    hooks.set_server_startup(startup)
