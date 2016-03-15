from collections import namedtuple

from outpost_data.core import image2
from outpost_data.core.builder2 import *
from outpost_data.outpost.lib.sprite_util import depth_stack


def get_pony_sprite():
    return SPRITE.get('pony')


Dir = namedtuple('Dir', ('idx', 'mirror'))

DIRS = [
        Dir(2, None),
        Dir(3, None),
        Dir(4, None),
        Dir(3, 1),
        Dir(2, 0),
        Dir(1, 7),
        Dir(0, None),
        Dir(1, None),
        ]

INV_DIRS = [None] * 5
for i, info in enumerate(DIRS):
    if info.mirror is None:
        INV_DIRS[info.idx] = i

Motion = namedtuple('Motion', ('name', 'row', 'base_col', 'len', 'fps'))

MOTIONS = [
        Motion('stand', 0, 0, 1,  1),
        Motion('walk',  1, 0, 6,  8),
        Motion('run',   3, 0, 6, 12),
        ]

SPRITE_SIZE = (96, 96)

BASES = {
        'E': ('base',),
        'P': ('backwing', 'base', 'frontwing'),
        'U': ('base', 'horn'),
        'A': ('backwing', 'base', 'horn', 'frontwing'),
        }

LAYER_DEPTHS = {
        'base': 100,
        'horn': 150,
        'frontwing': 150,
        'backwing': 50,
        }

def make_tribe_sheets(layer_imgs):
    '''Produce a combined sheet for each tribe, using a dict or function to
    obtain the four individual components base/horn/frontwing/backwing.'''
    if callable(layer_imgs):
        layer_imgs = {k: layer_imgs(k) for k in LAYER_DEPTHS.keys()}

    result = {}
    for tribe, layer_names in BASES.items():
        sheet = depth_stack([(layer_imgs[l], LAYER_DEPTHS[l]) for l in layer_names])
        result[tribe] = sheet
    return result

def standard_anims():
    '''Iterate over the standard pony animations (stand-0, walk-7, etc).'''
    for m in MOTIONS:
        for i in range(5):
            yield m, i, '%s-%d' % (m.name, INV_DIRS[i])


# Hat handling

HAT_SIZE = 64

def get_hat_box_pos(img):
    '''Compute the offset of the hat within a hat-box map.'''
    def f(raw):
        w, h = raw.size
        x = w // 2
        y = h // 2

        alpha = raw.split()[3]
        if alpha.getpixel((x, y)) == 0:
            return None

        while y < h - 1 and alpha.getpixel((x, y + 1)) != 0:
            y += 1
        while x > 0 and alpha.getpixel((x - 1, y)) != 0:
            x -= 1
        return (x, y + 1 - HAT_SIZE)

    return img.raw().compute(f)

def get_pony_hat_box(sex):
    pony = get_pony_sprite()
    part = pony.get_part('%s/_dummy' % sex)
    variant = part.get_variant('hat_box')
    return variant

def add_hat_layer(part, name, sex, sheet):
    '''Add a hat variant called `name` to `part`.  Use `sheet` as the hat image
    for ponies of the indicated sex.
    
    Note that `name` and `part` are used directly, with no modification based
    on `sex`.'''
    sheet = sheet.with_unit(HAT_SIZE)
    def f(_variant, anim, _frame_idx, img):
        idx = get_anim_facing(anim)
        hat = sheet.extract((idx, 0))
        hat_pos = get_hat_box_pos(img)
        return hat.pad(SPRITE_SIZE, offset=hat_pos)

    return part.add_derived_variant(name, get_pony_hat_box(sex), f)


# Map animation name to a direction

_ANIM_FACING_TABLE = {}

def register_anim_facing(name, facing):
    '''Register the direction of the named special animation.  `facing` should
    be a number 0..7 indicating the direction of the pony's head.'''
    _ANIM_FACING_TABLE[name] = facing

def get_anim_facing(anim):
    '''Get the direction of the pony's head in a particular animation.'''
    _, _, last = anim.name.rpartition('-')
    if last.isdigit():
        return DIRS[int(last)].idx
    else:
        return _ANIM_FACING_TABLE.get(full_name, 4)

