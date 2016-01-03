from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import tool as tool_, ward

def place(e, item, template=None, ignore_ward=False):
    pos = util.hit_tile(e)
    if not ignore_ward:
        ward.check(e, pos) 

    item = DATA.item(item)
    template = DATA.template(template if template is not None else item.name)

    if e.inv('main').count(item) == 0:
        return

    # If create_structure raises an exception, the item will not be used up.
    s = e.plane().create_structure(pos, template)
    e.inv('main').bulk_remove(item, 1)

    return s

def take(e, s, item, ignore_ward=False):
    if not ignore_ward:
        ward.check(e, s.pos())

    item = DATA.item(item)

    if e.inv('main').count_space(item) == 0:
        raise RuntimeError('no space for item in inventory')
    s.destroy()
    e.inv('main').bulk_add(item, 1)

def register(item, template=None, tool=None):
    item = DATA.item(item)
    template = DATA.template(template if template is not None else item.name)

    use.item(item)(lambda e, args: place(e, item, template))
    if tool is None:
        use.structure(template)(lambda e, s, args: take(e, s, item))
    else:
        tool_.handler(tool, template)(lambda e, s, args: take(e, s, item))
