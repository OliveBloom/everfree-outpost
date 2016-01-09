import builtins
import sys

from outpost_server import core
import outpost_server.core.data
import outpost_server.core.chat
import outpost_server.core.eval
import outpost_server.core.timer
import outpost_server.core.use


old_print = builtins.print
def err_print(*args, file=None, flush=True, **kwargs):
    old_print(*args, file=file or sys.stderr, flush=flush, **kwargs)
builtins.print = err_print

def startup(eng):
    pass


def client_login(eng, cid):
    from outpost_server.core.engine import ClientProxy
    c = ClientProxy(eng, cid)
    e = c.pawn()

    print('logged in: %s' % cid)
    if 'inited' not in e.extra():
        print('initing client')
        i_main = e.create_inv('main', 30)
        i_abil = e.create_inv('ability', 30)

        # TODO: kind of a hack.  need a proper core.lifecycle module that
        # outpost.* can hook into.
        if e.appearance() & (1 << 7):
            # unicorn
            i_abil.bulk_add('ability/light', 1)

        e.extra()['inited'] = True

    c.set_main_inventories(e.inv('main'), e.inv('ability'))

# TODO: get rid of all these hacks

# Fix this by letting terrain_gen set up `extra` directly (part of Bundle)
def hack_apply_structure_extras(eng, sid, k, v):
    from outpost_server.core.engine import StructureProxy
    s = StructureProxy(eng, sid)
    if k == 'loot':
        s.create_inv('main', 30)
        for part in v.split(','):
            if part == '':
                continue
            item, _, count_str = part.partition(':')
            s.inv().bulk_add(item, int(count_str))
    elif k == 'gem_puzzle_slot':
        from outpost_server.outpost.dungeon.gem_puzzle import init_slot
        puzzle_id, slot, init = v.split(',')
        init_slot(s, puzzle_id, int(slot), init)
    elif k == 'gem_puzzle_door':
        from outpost_server.outpost.dungeon.gem_puzzle import init_door
        puzzle_id = v
        init_door(s, puzzle_id)

    p = s.plane()

# Fix this once Bundle save/load code is working
def hack_run_load_hook(eng, sid):
    from outpost_server.core import state_machine, timer
    from outpost_server.core.engine import StructureProxy
    s = StructureProxy(eng, sid)
    t = s.extra().get('sm', {}).get('timer')
    if t is not None:
        when = t['when']
        cookie = timer.schedule(s.engine, when, 
                lambda eng: state_machine.callback(eng, sid, when))
        s.extra()['sm']['timer']['cookie'] = cookie


def init(storage, data, hooks):
    core.data.init(data)    # Must be first

    core.chat.init(hooks)
    core.eval.init(hooks)
    core.timer.init(hooks)
    core.use.init(hooks)

    hooks.server_startup(startup)
    hooks.client_login(client_login)
    hooks.hack_apply_structure_extras(hack_apply_structure_extras)
    hooks.hack_run_load_hook(hack_run_load_hook)

    import outpost_server.outpost
