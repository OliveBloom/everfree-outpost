from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import structure_items, tool


WORKBENCH_ITEM = DATA.item('workbench')
WORKBENCH_TEMPLATE = DATA.template('workbench')
WORKBENCH_ATTACHED_TEMPLATE = DATA.template('workbench/attached')

@use.structure('workbench')
@use.structure('workbench/attached')
def use_workbench(e, s, args):
    e.controller().open_crafting(s, e.inv('main'))

@tool.axe('workbench')
@tool.axe('workbench/attached')
def axe_workbench(e, s, args):
    structure_items.take(e, s, WORKBENCH_ITEM)

@use.item(WORKBENCH_ITEM)
def place_workbench(e, args):
    structure_items.place(e, WORKBENCH_ITEM, WORKBENCH_TEMPLATE)

    # TODO: attached workbench is currently disabled because crafting doesn't
    # support showing the same recipe for two different station templates
    #if structure_items.check_attachment(
    #        WORKBENCH_ATTACHED_TEMPLATE, e.plane(), util.hit_tile(e)):
    #    structure_items.place(e, WORKBENCH_ITEM, WORKBENCH_ATTACHED_TEMPLATE)
    #else:
    #    structure_items.place(e, WORKBENCH_ITEM, WORKBENCH_TEMPLATE)

#structure_items.register_attachment(WORKBENCH_ATTACHED_TEMPLATE, 'table')


@use.structure('furnace')
def anvil_structure(e, s, args):
    e.controller().open_crafting(s, e.inv('main'))

structure_items.register('furnace', tool='pickaxe')


@use.structure('anvil')
def anvil_structure(e, s, args):
    e.controller().open_crafting(s, e.inv('main'))

structure_items.register('anvil', tool='pickaxe')
