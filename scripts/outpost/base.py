from outpost_server.core import use

from outpost_server.outpost.lib import structure_items

@use.structure('anvil')
def anvil_structure(e, s, args):
    e.controller().open_crafting(s, e.inv('main'))

structure_items.register('anvil', tool='pickaxe')

