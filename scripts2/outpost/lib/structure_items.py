from outpost_server.core import util
from outpost_server.core.data import DATA

def place(e, item, template=None):
    item = DATA.item(item)
    template = DATA.template(template if template is not None else item.name)

    if e.inv('main').count(item) == 0:
        return

    # If create_structure raises an exception, the item will not be used up.
    s = e.plane().create_structure(util.hit_tile(e), template)
    e.inv('main').bulk_remove(item, 1)

def take(e, s, item):
    if e.inv('main').count_space(item) == 0:
        raise RuntimeError('no space for item in inventory')
    s.destroy()
    e.inv('main').bulk_add(item, 1)
