from outpost_data.core.builder2 import EXTRA
from outpost_data.outpost.lib import pony_sprite


def pony(maps):
    return maps.sprites['pony']

def gen_default_anim(maps):
    return pony(maps).get_anim('stand-0').id

def gen_editor_anim(maps):
    return pony(maps).get_anim('stand-4').id

def gen_physics_anim_table(maps):
    SPEED_NAMES = ('stand', 'walk', None, 'run')
    table = []
    for speed in SPEED_NAMES:
        if speed is None:
            table.append(None)
            continue

        table.append([pony(maps).get_anim('%s-%d' % (speed, dir_)).id
            for dir_ in range(8)])

    return table

def gen_anim_dir_table(maps):
    dct = {}
    #for anim in pony(maps).iter_anims():
        #dct[anim.id] = pony_sprite.get_anim_facing(anim)

    # TODO: There's no good reason to restrict this to physics anims, except
    # that's how the current hack in client/js/physics.js distinguishes physics
    # anims from special ones.
    for motion in ('stand', 'walk', 'run'):
        for dir_ in range(8):
            anim = pony(maps).get_anim('%s-%d' % (motion, dir_))
            dct[anim.id] = dir_
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
    for bits in range(8):
        # This mimics the logic in client/js/graphics/appearance/pony.js
        tribe_idx = bits & 3
        tribe = ('E', 'P', 'U', 'A')[tribe_idx]
        stallion_bit = (bits >> 2) & 1
        sex = ('f', 'm')[stallion_bit]

        result.append(pony(maps).get_part('%s/base' % sex).get_variant(tribe).id)
    return result

def init():
    EXTRA.new('default_anim').func(gen_default_anim)
    EXTRA.new('editor_anim').func(gen_editor_anim)
    EXTRA.new('physics_anim_table').func(gen_physics_anim_table)
    EXTRA.new('anim_dir_table').func(gen_anim_dir_table)
    EXTRA.new('pony_slot_table').func(gen_pony_slot_table)
    EXTRA.new('pony_bases_table').func(gen_pony_bases_table)
