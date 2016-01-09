import random

from outpost_server.core import alias, use, util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import structure_items, tool, ward

structure_items.register('bookshelf', 'bookshelf/0')
structure_items.register_attachment('bookshelf/0', 'wall/horiz')
BOOKSHELF = [DATA.template('bookshelf/%d' % i) for i in range(3)]
BOOK = DATA.item('book')

@use.structure(BOOKSHELF[2])
def use_bookshelf_2(e, s, args):
    if e.inv().count_space(BOOK) == 0:
        return
    s.replace(BOOKSHELF[1])
    e.inv().bulk_add(BOOK, 1)

@use.structure(BOOKSHELF[1])
def use_bookshelf_1(e, s, args):
    if e.inv().count_space(BOOK) == 0:
        return
    s.replace(BOOKSHELF[0])
    e.inv().bulk_add(BOOK, 1)

@use.item(BOOK)
def use_book(e, args):
    s = util.hit_structure(e)
    if s.template() not in BOOKSHELF:
        return
    idx = BOOKSHELF.index(s.template())
    if idx < 2:
        e.inv().bulk_remove(BOOK, 1)
        s.replace(BOOKSHELF[idx + 1])



TREE = DATA.template('tree/v0')
STUMP = DATA.template('stump')
WOOD = DATA.item('wood')

alias.register_template('tree/v1', TREE)

@use.structure(TREE)
@use.structure(STUMP)
def use_tree(e, s, args):
    e.inv().bulk_add(WOOD, 2)

@tool.axe(TREE)
def axe_tree(e, s, args):
    ward.check(e, s.pos())
    s.replace(STUMP)
    e.inv().bulk_add(WOOD, 15)

@tool.axe(STUMP)
def axe_stump(e, s, args):
    ward.check(e, s.pos())
    s.destroy()
    e.inv().bulk_add(WOOD, 5)


ROCK = DATA.template('rock')
STONE = DATA.item('stone')
CRYSTAL = DATA.item('crystal')

@use.structure(ROCK)
def use_rock(e, s, args):
    e.inv().bulk_add(STONE, 2)

@tool.pickaxe(ROCK)
def pickaxe_rock(e, s, args):
    ward.check(e, s.pos())
    s.destroy()
    e.inv().bulk_add(STONE, 20)
    if random.randrange(10) < 5:
        e.inv().bulk_add(CRYSTAL, 1)
