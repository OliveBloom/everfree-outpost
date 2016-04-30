from outpost_server.core import use
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import appearance


# TODO: most of this is copied from hat.py

REMOVE_SOCKS = DATA.item('ability/remove_socks')

def wear_socks(e, item, name):
    item = DATA.item(item)
    if e.inv('ability').count(REMOVE_SOCKS) != 0:
        # Character is already wearing a socks.
        e.controller().send_message('You are already wearing socks!')
        return

    e.inv('main').bulk_remove(item, 1)
    e.inv('ability').bulk_add(REMOVE_SOCKS, 1)
    e.extra()['socks_type'] = item.name

    appearance.set_socks(e, name)

def register_socks(item, name):
    @use.item(item)
    def use_socks(e, args):
        wear_socks(e, item, name)

COLORS = [
        'red',
        'orange',
        'yellow',
        'green',
        'blue',
        'purple',
        'white',
        'black',
        ]
for c in COLORS:
    register_socks('socks/solid/%s' % c, 'socks/solid/%s' % c)

@use.ability(REMOVE_SOCKS)
def remove_socks(e, args):
    item = DATA.item(e.extra()['socks_type'])

    if e.inv('main').count_space(item) == 0:
        # No room for socks in inventory.
        e.controller().send_message('No space for socks in inventory')
        return

    e.inv('main').bulk_add(item, 1)
    e.inv('ability').bulk_remove(REMOVE_SOCKS, 1)
    del e.extra()['socks_type']
    appearance.set_socks(e, None)
