from outpost_server.core import inventory_hooks, use
from outpost_server.core.data import DATA
from outpost_server.core.engine import StructureProxy
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



def register_celestial_container(item, template):
    item = DATA.item(item)
    template = DATA.template(template)

    @use.item(item)
    def use_item(e, args):
        structure_items.place(e, item, template)

    @use.structure(template)
    def use_structure(e, s, args):
        ward.check(e, s.pos())
        inv = e.inv('celestial')
        if inv is None:
            inv = e.create_inv('celestial', 30)
        e.controller().open_container(e.inv(), inv)

    @tool.axe(template)
    def axe_structure(e, s, args):
        structure_items.take(e, s, item)

register_celestial_container('chest/celestial', 'chest/celestial')



CRATE_BASE = DATA.template('crate')
CRATE_VARIANTS = {}
CRATE_SIZE = 30
CRATE_ITEM = DATA.item('crate')

@use.item(CRATE_ITEM)
def crate_use_item(e, args):
    s = structure_items.place(e, CRATE_ITEM, CRATE_BASE)
    i = s.create_inv('main', CRATE_SIZE)
    i.set_special('crate', s.id)

@use.structure(CRATE_BASE)
def crate_use_structure(e, s, args):
    ward.check(e, s.pos())
    e.controller().open_container(e.inv(), s.inv())

@tool.axe(CRATE_BASE)
def crate_axe(e, s, args):
    structure_items.take(e, s, CRATE_ITEM)

def register_crate_variant(name, item_name=None):
    if item_name is None:
        item_name = name
    t = DATA.template('crate/%s' % name)
    i = DATA.item(item_name)
    assert i not in CRATE_VARIANTS, 'duplicate crate variant for item %r' % i
    CRATE_VARIANTS[i] = t

    use.structure(t)(crate_use_structure)
    tool.axe(t)(crate_axe)

@inventory_hooks.register('crate')
def crate_hook(i, sid):
    s = StructureProxy(i._eng, sid)
    item = i.slot_item(0)
    variant = CRATE_VARIANTS.get(item, CRATE_BASE)
    if s.template != variant:
        s.replace(variant)
