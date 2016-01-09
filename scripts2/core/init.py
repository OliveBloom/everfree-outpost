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

def hack_set_main_inventories(eng, cid, item_iid, ability_iid):
    from outpost_server.core.engine import ClientProxy
    c = ClientProxy(eng, cid)
    inv = c.pawn().extra().setdefault('inv', {})
    inv['main'] = item_iid
    inv['ability'] = ability_iid

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
    # Must be first
    core.data.init(data)

    core.chat.init(hooks)
    core.eval.init(hooks)
    core.timer.init(hooks)
    core.use.init(hooks)

    hooks.server_startup(startup)
    hooks.hack_set_main_inventories(hack_set_main_inventories)
    hooks.hack_apply_structure_extras(hack_apply_structure_extras)

    import outpost_server.outpost
