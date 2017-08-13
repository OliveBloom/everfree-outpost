import random

from outpost_server.core import alias, use, util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import structure_items, timed_action, tool, ward
from outpost_server.outpost.lib.energy import energy_cost



TREE = DATA.template('tree/v0')
STUMP = DATA.template('stump')
WOOD = DATA.item('wood')

alias.register_template('tree/v1', TREE)

@use.structure(TREE)
@use.structure(STUMP)
@energy_cost(2)
@timed_action.action('activity//activity/kick', 1000)
def use_tree(e, s, args):
    e.inv().bulk_add(WOOD, 2)

@tool.axe(TREE)
@timed_action.action('activity//item/axe/stone', check=tool.default_check(1000))
def axe_tree(e, s, args):
    ward.check(e, s.pos())
    s.replace(STUMP)
    e.inv().bulk_add(WOOD, 40)

@tool.axe(STUMP)
@timed_action.action('activity//item/axe/stone', check=tool.default_check(1000))
def axe_stump(e, s, args):
    ward.check(e, s.pos())
    s.destroy()
    e.inv().bulk_add(WOOD, 10)


ROCK = DATA.template('rock')
STONE = DATA.item('stone')
CRYSTAL = DATA.item('crystal')

@use.structure(ROCK)
@energy_cost(2)
@timed_action.action('activity//activity/kick', 1000)
def use_rock(e, s, args):
    e.inv().bulk_add(STONE, 2)

@tool.pickaxe(ROCK)
@timed_action.action('activity//item/pick/stone', check=tool.default_check(1000))
def pickaxe_rock(e, s, args):
    ward.check(e, s.pos())
    s.destroy()
    e.inv().bulk_add(STONE, 50)
    e.inv().bulk_add(CRYSTAL, random.randrange(0, 3))
