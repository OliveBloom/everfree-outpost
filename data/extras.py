from outpost_data.core.builder2 import EXTRA
from outpost_data.outpost.lib import pony_sprite


def pony(maps):
    return maps.sprites['pony']

def gen_default_anim(maps):
    return pony(maps).get_anim('stand-0').id

def gen_editor_anim(maps):
    return pony(maps).get_anim('stand-2').id

def gen_activity_bubble_graphics(maps):
    return maps.sprites['activity_bubble'].get_graphics('default', 'default').id

def gen_physics_anim_table(maps):
    SPEED_NAMES = ('stand', 'walk', None, 'run')
    table = []
    for speed in SPEED_NAMES:
        if speed is None:
            table.append(None)
            continue

        table.append([pony(maps).get_anim('%s-%d' % (speed, dir_)).id
            for dir_ in (0, 0, 1, 2, 2, 2, 3, 0)])

    return table

def gen_anim_dir_table(maps):
    dct = {}
    #for anim in pony(maps).iter_anims():
        #dct[anim.id] = pony_sprite.get_anim_facing(anim)

    # TODO: There's no good reason to restrict this to physics anims, except
    # that's how the current hack in client/js/physics.js distinguishes physics
    # anims from special ones.
    # TODO: the reasoning above may be out of date now (but nothing's broken yet...)
    for motion in ('stand', 'walk', 'run'):
        for dir_ in range(4):
            anim = pony(maps).get_anim('%s-%d' % (motion, dir_))
            dct[anim.id] = dir_ * 2
    return dct

def gen_pony_slot_table(maps):
    result = []
    for sex in ('f', 'm'):
        parts = {}
        for part in ('base', 'mane', 'tail', 'eyes', 'equip0', 'equip1', 'equip2'):
            parts[part] = pony(maps).get_part('%s/%s' % (sex, part)).id
        result.append(parts)
    return result

def gen_pony_bases_table(maps):
    result = []
    for stallion_bit in range(2):
        row = []
        for tribe_idx in range(4):
            # This mimics the logic in client/js/graphics/appearance/pony.js
            tribe = ('E', 'P', 'U', 'A')[tribe_idx]
            sex = ('f', 'm')[stallion_bit]

            row.append(pony(maps).get_part('%s/base' % sex).get_variant(tribe).local_id)
        result.append(row)
    return result

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

def gen_pony_layer_table(maps):
    p = pony(maps)
    result = []
    for layer in ('base', 'horn', 'frontwing', 'backwing', 'eyes1',
            'mane/1', 'mane/2', 'mane/3',
            'tail/1', 'tail/2', 'tail/3',
            'hat/party', 'hat/santa', 'hat/witch', 'hat/explorer') + \
            tuple('socks/solid/%s' % c for c in COLORS):
        for sex in 'fm':
            try:
                l = p.get_layer('%s/%s' % (sex, layer))
                result.append(l.id)
            except KeyError:
                result.append(255)
    return result

def init():
    EXTRA.new('default_anim').func(gen_default_anim)
    EXTRA.new('editor_anim').func(gen_editor_anim)
    EXTRA.new('activity_bubble_graphics').func(gen_activity_bubble_graphics)
    EXTRA.new('physics_anim_table').func(gen_physics_anim_table)
    EXTRA.new('anim_dir_table').func(gen_anim_dir_table)
    EXTRA.new('pony_layer_table').func(gen_pony_layer_table)
