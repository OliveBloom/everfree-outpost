from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import door

door.register('dungeon/door/key', 500)
door.register('dungeon/door/puzzle', 500)

KEY = DATA.item('key')
MASTER_KEY = DATA.item('key/master')
KEY_DOOR = DATA.template('dungeon/door/key/closed')

@use.structure(KEY_DOOR)
def use_key_door(e, s, args):
    if e.inv().count(MASTER_KEY) > 0:
        e.controller().send_message('Opened door using skeleton key')
        door.open(s)
    elif e.inv().count(KEY) > 0:
        e.inv().bulk_remove(KEY, 1)
        door.open(s)
    else:
        e.controller().send_message('You need a key to open this door.')

@use.item(KEY)
def use_key(e, args):
    s = util.hit_structure(e)
    if s is not None and s.template() is KEY_DOOR:
        e.inv().bulk_remove(KEY, 1)
        door.open(s)

@use.item(MASTER_KEY)
def use_master_key(e, args):
    s = util.hit_structure(e)
    if s is not None and s.template() is KEY_DOOR:
        door.open(s)
