import builtins
import sys

from outpost_server import core
import outpost_server.core.data
import outpost_server.core.chat
import outpost_server.core.eval
import outpost_server.core.use


old_print = builtins.print
def err_print(*args, file=None, flush=True, **kwargs):
    old_print(*args, file=file or sys.stderr, flush=flush, **kwargs)
builtins.print = err_print

def startup(eng):
    from outpost_server.core.data import DATA
    sys.stderr.write('hello %s\n' % eng)
    sys.stderr.write('hello %s\n' % DATA.recipe('anvil'))
    sys.stderr.write('hello %s\n' % eng.now())
    sys.stderr.flush()

def init(storage, data, hooks):
    core.data.init(data)
    core.chat.init(hooks)
    core.eval.init(hooks)
    core.use.init(hooks)

    hooks.server_startup(startup)

    import outpost_server.outpost
