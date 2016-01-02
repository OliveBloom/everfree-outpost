from outpost_server.core import use

from outpost_server.outpost.lib import structure_items

@use.structure('anvil')
def anvil_structure(c, s, args):
    c.open_crafting(s, c.pawn().inv('main'))

@use.item('anvil')
def anvil_item(c, args):
    structure_items.place(c.pawn(), 'anvil')

