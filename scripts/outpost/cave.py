from outpost_server.core import util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import tool
from outpost_server.outpost.lib import ward

@tool.handler('pickaxe', 'cave_junk/0')
@tool.handler('pickaxe', 'cave_junk/1')
@tool.handler('pickaxe', 'cave_junk/2')
def cave_junk(e, s, args):
    ward.check(e, s.pos())
    s.destroy()
    e.inv('main').bulk_add(DATA.item('stone'), 2)

def mine_wall(e, args):
    # TODO: check forest
    pos = util.hit_tile(e)
    ward.check(e, pos)

    if e.plane().set_cave(pos):
        e.inv('main').bulk_remove(DATA.item('pick'), 1)
        e.inv('main').bulk_add(DATA.item('stone'), 20)
