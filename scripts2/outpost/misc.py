from outpost_server.core import use, util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import structure_items

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
