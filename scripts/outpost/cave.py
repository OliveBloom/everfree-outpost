from outpost_server.core import util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import timed_action, tool, util as util2, ward

@tool.handler('pickaxe', 'cave_junk/0')
@tool.handler('pickaxe', 'cave_junk/1')
@tool.handler('pickaxe', 'cave_junk/2')
@timed_action.action('activity//item/pick', check=tool.default_check(1000))
def cave_junk(e, s, args):
    ward.check(e, s.pos())
    s.destroy()
    e.inv('main').bulk_add(DATA.item('stone'), 2)


def mine_wall(e, args):
    if not e.plane().is_cave(util.hit_tile(e)):
        return

    ward.check(e, util.hit_tile(e))
    util2.forest_check(e)

    if args.level < 1:
        e.controller().send_message('This pick is not strong enough.')
        return

    delay = 5000 if args.level == 1 else 3000
    timed_action.run(mine_wall_impl, 'activity//item/pick', delay, e)

def mine_wall_impl(e):
    pos = util.hit_tile(e)

    if e.plane().set_cave(pos):
        e.inv('main').bulk_add(DATA.item('stone'), 20)


@tool.pickaxe('ore_vein/copper')
@timed_action.action('activity//item/pick', check=tool.default_check(1000))
def pickaxe_copper(e, s, args):
    ward.check(e, s.pos())
    if e.inv().count_space('ore/copper') == 0:
        return
    s.destroy()
    e.inv().bulk_add('ore/copper', 1)

@tool.pickaxe('ore_vein/iron')
@timed_action.action('activity//item/pick', check=tool.default_check(1000))
def pickaxe_iron(e, s, args):
    ward.check(e, s.pos())
    if e.inv().count_space('ore/iron') == 0:
        return
    s.destroy()
    e.inv().bulk_add('ore/iron', 1)
