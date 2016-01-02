from outpost_server.core import use

from outpost_server.outpost.lib import structure_items, tool

@use.structure('anvil')
def anvil_structure(e, s, args):
    e.controller().open_crafting(s, e.inv('main'))

@use.item('anvil')
def anvil_item(e, args):
    structure_items.place(e, 'anvil')

@tool.pickaxe('anvil')
def anvil_pickaxe(e, s, args):
    structure_items.take(e, s, 'anvil')

