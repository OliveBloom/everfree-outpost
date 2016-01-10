from outpost_server.core import use
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import structure_items, tool, ward

def register_container(item, template, size):
    item = DATA.item(item)
    template = DATA.template(template)

    @use.item(item)
    def use_item(e, args):
        s = structure_items.place(e, item, template)
        s.create_inv('main', size)

    @use.structure(template)
    def use_structure(e, s, args):
        ward.check(e, s.pos())
        e.controller().open_container(e.inv(), s.inv())

    @tool.axe(template)
    def axe_structure(e, s, args):
        structure_items.take(e, s, item)

register_container('chest', 'chest', 30)
register_container('barrel', 'barrel', 30)
register_container('cabinets', 'cabinets', 30)
structure_items.register_attachment('cabinets', 'wall/horiz')

