from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost import cave
from outpost_server.outpost.lib import mallet, tool

def lua_fallback(item, e, args):
    print('PASS THROUGH: item(tool)', DATA.item(item))
    cid = e.controller().id
    item_id = DATA.item(item).id
    e._eng.script_cb_use_item(cid, item_id, args)

@use.item('axe')
def axe(e, args):
    ok = tool.use(e, 'axe')
    # TODO: hack for lua passthrough - remove
    if ok is False:
        lua_fallback('axe', e, args)

@use.item('pick')
def pickaxe(e, args):
    if util.hit_structure(e) is None:
        cave.mine_wall(e, args)
    else:
        ok = tool.use(e, 'pickaxe')
        # TODO: hack for lua passthrough - remove
        if ok is False:
            lua_fallback('pick', e, args)

use.item('mallet')(mallet.use)
