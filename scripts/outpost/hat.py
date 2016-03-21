from outpost_server.core import use
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import appearance


REMOVE_HAT = DATA.item('ability/remove_hat')

def wear_hat(e, item, name):
    item = DATA.item(item)
    if e.inv('ability').count(REMOVE_HAT) != 0:
        # Character is already wearing a hat.
        e.controller().send_message('You are already wearing a hat!')
        return

    e.inv('main').bulk_remove(item, 1)
    e.inv('ability').bulk_add(REMOVE_HAT, 1)
    e.extra()['hat_type'] = item.name

    appearance.set_hat(e, name)

def register_hat(item, name):
    @use.item(item)
    def use_hat(e, args):
        wear_hat(e, item, name)

register_hat('hat', 'hat/witch')
register_hat('party_hat','hat/party')
register_hat('santa_hat', 'hat/santa')

@use.ability(REMOVE_HAT)
def remove_hat(e, args):
    item = DATA.item(e.extra()['hat_type'])

    if e.inv('main').count_space(item) == 0:
        # No room for hat in inventory.
        e.controller().send_message('No space for hat in inventory')
        return

    e.inv('main').bulk_add(item, 1)
    e.inv('ability').bulk_remove(REMOVE_HAT, 1)
    del e.extra()['hat_type']
    appearance.set_hat(e, None)
