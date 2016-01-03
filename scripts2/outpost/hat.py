from outpost_server.core import use
from outpost_server.core.data import DATA


HAT_BASE = 18
HAT_BITS = 4
HAT_MASK = ((1 << HAT_BITS) - 1) << HAT_BASE

def set_hat(e, hat_idx):
    old_a = e.appearance()
    new_a = (old_a & ~HAT_MASK) | (hat_idx << HAT_BASE)
    e.set_appearance(new_a)

def wear_hat(e, item, idx):
    item = DATA.item(item)
    remove_hat = DATA.item('ability/remove_hat')
    if e.inv('ability').count(remove_hat) != 0:
        # Character is already wearing a hat.
        return

    e.inv('main').bulk_remove(item, 1)
    e.inv('ability').bulk_add(remove_hat, 1)
    e.extra()['hat_type'] = item.name
    set_hat(e, idx)

def register_hat(item, idx):
    @use.item(item)
    def use_hat(e, args):
        wear_hat(e, item, idx)

register_hat('hat', 1)
register_hat('party_hat', 1)
register_hat('santa_hat', 1)

@use.ability('ability/remove_hat')
def remove_hat(e, args):
    item = DATA.item(e.extra()['hat_type'])

    if e.inv('main').count_space(item) == 0:
        # No room for hat in inventory.
        return

    e.inv('main').bulk_add(item, 1)
    e.inv('ability').bulk_remove(DATA.item('ability/remove_hat'), 1)
    del e.extra()['hat_type']
    set_hat(e, 0)
