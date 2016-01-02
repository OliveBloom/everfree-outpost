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

def hack_set_main_inventories(eng, cid, item_iid, ability_iid):
    from outpost_server.core.engine import ClientProxy
    c = ClientProxy(eng, cid)
    inv = c.pawn().extra().setdefault('inv', {})
    inv['main'] = item_iid
    inv['ability'] = ability_iid
    print('set main invs to', inv.copy())
    print(c.pawn().extra().copy())
    print(c.pawn().extra()['inv'].copy())

def init(storage, data, hooks):
    core.data.init(data)
    core.chat.init(hooks)
    core.eval.init(hooks)
    core.use.init(hooks)

    hooks.server_startup(startup)
    hooks.hack_set_main_inventories(hack_set_main_inventories)

    import outpost_server.outpost
