from outpost_server.core.data import DATA


def _mask(base, bits):
    return ((1 << bits) - 1) << base


TRIBE_BASE = 6
TRIBE_BITS = 2
TRIBE_MASK = _mask(TRIBE_BASE, TRIBE_BITS)

TRIBE_NAME = 'EPUA'
TRIBE_MAP = {x: i for i,x in enumerate(TRIBE_NAME)}

def set_tribe(e, name):
    old_a = e.appearance()
    new_a = (old_a & ~TRIBE_MASK) | (TRIBE_MAP[name] << TRIBE_BASE)
    e.set_appearance(new_a)

def get_tribe(e):
    a = e.appearance()
    return TRIBE_NAME[(a & TRIBE_MASK) >> TRIBE_BASE]

def is_tribe(e, tribe):
    t = get_tribe(e)
    return t == tribe or t == 'A'


STALLION_BIT = 1 << 8

def set_stallion(e, stallion):
    old_a = e.appearance()
    if stallion:
        new_a = old_a | STALLION_BIT
    else:
        new_a = old_a & ~STALLION_BIT
    e.set_appearance(new_a)

def is_stallion(e):
    a = e.appearance()
    return bool(a & STALLION_BIT)


HAT_BASE = 18
HAT_BITS = 4
HAT_MASK = _mask(HAT_BASE, HAT_BITS)

def set_hat(e, name):
    old_a = e.appearance()

    sex = 'm' if old_a & STALLION_BIT else 'f'
    if name is not None:
        hat_id = DATA.sprite_part('pony//%s/equip0' % sex).variant_id(name)
    else:
        hat_id = 0

    new_a = (old_a & ~HAT_MASK) | (hat_id << HAT_BASE)
    e.set_appearance(new_a)


LIGHT_BASE = 9
LIGHT_MASK = 1 << LIGHT_BASE

def set_light(e, flag):
    old_a = e.appearance()
    if flag:
        new_a = old_a | LIGHT_MASK
    else:
        new_a = old_a & ~LIGHT_MASK
    e.set_appearance(new_a)

def get_light(e):
    return (e.appearance() & LIGHT_MASK) != 0
