import builtins
import importlib
import os
import sys

from outpost_server import core
import outpost_server.core.data
import outpost_server.core.chat
import outpost_server.core.eval
import outpost_server.core.import_hooks
import outpost_server.core.state_machine
import outpost_server.core.timer
import outpost_server.core.use


old_print = builtins.print
def err_print(*args, file=None, flush=True, **kwargs):
    old_print(*args, file=file or sys.stderr, flush=flush, **kwargs)
builtins.print = err_print

def startup(eng):
    pass


def client_login(eng, cid):
    from outpost_server.core.data import DATA
    from outpost_server.core.engine import ClientProxy, Walk

    c = ClientProxy(eng, cid)
    e = c.pawn()

    c.set_main_inventories(e.inv('main'), e.inv('ability'))

    if 'work_timer' in e.extra():
        del e.extra()['work_timer']
        e.set_activity(Work(
            DATA.animation_id('pony//stand-0'),
            DATA.animation_id('activity//none')))
        e.set_activity(Walk())

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

def init(storage, data, hooks):
    core.data.init(data)    # Must be first

    core.chat.init(hooks)
    core.eval.init(hooks)
    core.import_hooks.init(hooks)
    core.timer.init(hooks)
    core.use.init(hooks)

    hooks.server_startup(startup)
    hooks.client_login(client_login)
    hooks.hack_apply_structure_extras(hack_apply_structure_extras)

    for d in outpost_server.__path__:
        for m in os.listdir(d):
            if m.endswith('.py'):
                m = m[:-len('.py')]
            if m in ('boot', 'core'):
                continue
            print('Loading %s...' % m)
            importlib.import_module('outpost_server.%s' % m)
