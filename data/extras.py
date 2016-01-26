from outpost_data.core.builder2 import EXTRA


def gen_default_anim(maps):
    return maps.animations['pony/stand-0']

def gen_editor_anim(maps):
    return maps.animations['pony/stand-4']

def gen_physics_anim_table(maps):
    SPEED_NAMES = ('stand', 'walk', None, 'run')
    table = []
    for speed in SPEED_NAMES:
        if speed is None:
            table.append(None)
            continue

        table.append([maps.animations['pony/%s-%d' % (speed, dir_)]
            for dir_ in range(8)])

    return table

def gen_anim_dir_table(maps):
    dct = {}
    for speed in ('stand', 'run', 'walk'):
        for dir_ in range(8):
            anim_id = maps.animations['pony/%s-%d' % (speed, dir_)]
            dct[anim_id] = dir_
    return dct

def gen_pony_slot_table(maps):
    result = []
    for sex in ('f', 'm'):
        parts = {}
        for part in ('base', 'mane', 'tail', 'eyes', 'equip0', 'equip1', 'equip2'):
            # TODO: replace .get() with []
            parts[part] = maps.attach_slots.get('pony/%s/%s' % (sex, part))
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

        # TODO: replace .get() with []
        result.append(maps.attachments_by_slot.get('pony/%s/base' % sex, {}).get(tribe))
    return result

def init():
    EXTRA.new('default_anim').func(gen_default_anim)
    EXTRA.new('editor_anim').func(gen_editor_anim)
    EXTRA.new('physics_anim_table').func(gen_physics_anim_table)
    EXTRA.new('anim_dir_table').func(gen_anim_dir_table)
    EXTRA.new('pony_slot_table').func(gen_pony_slot_table)
    EXTRA.new('pony_bases_table').func(gen_pony_bases_table)
