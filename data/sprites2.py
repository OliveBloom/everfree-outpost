from collections import namedtuple

from outpost_data.core import image2
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes

import PIL.Image
import PIL.ImageChops


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

def _set_depth(img, depth):
    old_alpha = img.split()[3]
    mask = PIL.Image.new('L', img.size, depth)
    img.putalpha(PIL.ImageChops.darker(mask, old_alpha))
    return img

def set_depth(img, depth):
    return img.modify(f=lambda raw: _set_depth(raw, depth),
            desc=('sprites2.set_depth', depth))

def _depth_stack(base, layer, depth):
    # Original alpha channel of `layer` is a mask indicating which parts are
    # covered by `layer`.
    mask = layer.split()[3].copy()
    # Replace alpha channel with constant depth
    layer.putalpha(PIL.Image.new('L', layer.size, depth))
    # Overwrite `base` with relevant parts of `layer`, including the new depth.
    base.paste(layer, (0, 0), mask)
    return base

def depth_stack(base, layer, depth):
    return base.fold((layer,),
            f=lambda b, l: _depth_stack(b, l, depth),
            desc=('sprites2.depth_stack', depth))

def make_base_sheet(load, ms, i, layers):
    acc = None
    for l in layers:
        img = load('base/%s/%s-%d-%s.png' % (ms, ms, i, l))
        if acc is None:
            acc = set_depth(img, LAYER_DEPTHS[l])
        else:
            acc = depth_stack(acc, img, LAYER_DEPTHS[l])
    return acc


def init():
    pony = SPRITE.new('pony')
    load = loader('sprites', unit=SPRITE_SIZE)

    # Define animations
    for m in MOTIONS:
        for i, d in enumerate(DIRS):
            if d.mirror is None:
                pony.anim('%s-%d' % (m.name, i), m.len, m.fps)
            else:
                pony.mirror_anim('%s-%d' % (m.name, i), '%s-%d' % (m.name, d.mirror))

    # Define layers
    for sex, ms in (('f', 'mare'), ('m', 'stallion')):
        pony.layer('%s/base' % sex, SPRITE_SIZE)

        for i in range(5):
            sheet = make_base_sheet(load, ms, i, BASES['A'])

            for m in MOTIONS:
                row = sheet.extract((m.base_col, m.row), size=(m.len, 1))
                anim = row.sheet_to_anim((1, 1), m.fps)
                pony.add_graphics('%s/base' % sex, '%s-%d' % (m.name, INV_DIRS[i]), anim)




