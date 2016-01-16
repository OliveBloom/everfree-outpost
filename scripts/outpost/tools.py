from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost import cave
from outpost_server.outpost.lib import mallet, tool, ward

@use.item('axe')
def axe(e, args):
    tool.use(e, 'axe')

@use.item('pick')
def pickaxe(e, args):
    if util.hit_structure(e) is None:
        cave.mine_wall(e, args)
    else:
        tool.use(e, 'pickaxe')

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
    elif block.name.startswith('grass/'):
        e.plane().set_farmland(pos)
