from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost import cave
from outpost_server.outpost.lib import mallet, tool, ward

@use.item('axe/stone')
def axe(e, args):
    tool.use(e, tool.Args('axe', 0))

@use.item('axe/copper')
def axe(e, args):
    tool.use(e, tool.Args('axe', 1))

@use.item('axe/iron')
def axe(e, args):
    tool.use(e, tool.Args('axe', 2))



@use.item('pick/stone')
def pickaxe(e, args):
    args = tool.Args('pickaxe', 0)
    if util.hit_structure(e) is None:
        cave.mine_wall(e, args)
    else:
        tool.use(e, args)

@use.item('pick/copper')
def pickaxe(e, args):
    args = tool.Args('pickaxe', 1)
    if util.hit_structure(e) is None:
        cave.mine_wall(e, args)
    else:
        tool.use(e, args)

@use.item('pick/iron')
def pickaxe(e, args):
    args = tool.Args('pickaxe', 2)
    if util.hit_structure(e) is None:
        cave.mine_wall(e, args)
    else:
        tool.use(e, args)


use.item('mallet')(mallet.use)

@use.item('shovel')
def shovel(e, args):
    pos = util.hit_tile(e)

    ward.check(e, pos)
    if e.plane().find_structure_at_point(pos) is not None:
        return

    block = e.plane().get_block(pos)
    if block.name.startswith('farmland/'):
        e.plane().clear_farmland(pos)
    elif block.name.startswith('terrain/gggg/v'):
        e.plane().set_farmland(pos)
