from collections import namedtuple

from outpost_data.core import image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
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

def depth_stack(img_depths):
    imgs = tuple(i for i, d in img_depths)
    depths = tuple(d for i, d in img_depths)
    assert len(imgs) > 0

    def f(*args):
        acc = PIL.Image.new('RGBA', args[0].size)
        for img, depth in zip(args, depths):
            # Extract original alpha channel
            mask = img.split()[3].copy()
            # Set alpha to depth, uniformly
            img.putalpha(PIL.Image.new('L', img.size, depth))
            # Overwrite `acc` with `img` (including alpha), filtered by `mask`
            acc.paste(img, (0, 0), mask)
        return acc

    return imgs[0].fold(imgs[1:], f=f, desc=('sprites2.depth_stack', depths))


def make_tribe_sheets(layer_imgs):
    if callable(layer_imgs):
        layer_imgs = {k: layer_imgs(k) for k in LAYER_DEPTHS.keys()}

    result = {}
    for tribe, layer_names in BASES.items():
        sheet = depth_stack([(layer_imgs[l], LAYER_DEPTHS[l]) for l in layer_names])
        result[tribe] = sheet
    return result

def standard_anims():
    for m in MOTIONS:
        for i in range(5):
            yield m, i, '%s-%d' % (m.name, INV_DIRS[i])

def standard_manes_tails():
    for kind in ('mane', 'tail'):
        for i in (1, 2, 3):
            yield (kind, i)


HAT_SIZE = 64

def get_hat_box_pos(img):
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

_ANIM_FACING_TABLE = {}

def register_anim_facing(name, facing):
    _ANIM_FACING_TABLE[name] = facing

def get_anim_facing(anim):
    _, _, last = anim.name.rpartition('-')
    if last.isdigit():
        return DIRS[int(last)].idx
    else:
        return _ANIM_FACING_TABLE.get(full_name, 4)

def add_hat_layer0(sprite, name, hat_name, sheet):
    sheet = sheet.with_unit(HAT_SIZE)
    def f(anim, img):
        idx = get_anim_facing(anim)
        hat = sheet.extract((idx, 0))
        hat_pos = get_hat_box_pos(img)
        return hat.pad(SPRITE_SIZE, offset=hat_pos)

    sprite.derived_layer(name, hat_name, f)


def get_pony_sprite():
    return SPRITE.get('pony')

def get_pony_hat_box(sex):
    pony = get_pony_sprite()
    part = pony.get_part('%s/_dummy' % sex)
    variant = part.get_variant('hat_box')
    return variant

def add_hat_layer(part, name, sex, sheet):
    sheet = sheet.with_unit(HAT_SIZE)
    def f(_variant, anim, _frame_idx, img):
        idx = get_anim_facing(anim)
        hat = sheet.extract((idx, 0))
        hat_pos = get_hat_box_pos(img)
        return hat.pad(SPRITE_SIZE, offset=hat_pos)

    return part.add_derived_variant(name, get_pony_hat_box(sex), f)



def init():
    pony = SPRITE.new('pony', SPRITE_SIZE)
    load = loader('sprites', unit=SPRITE_SIZE)
    load1 = loader('sprites')

    # Define animations
    for m in MOTIONS:
        dct = {}
        for i, d in enumerate(DIRS):
            if d.mirror is None:
                dct[i] = pony.add_anim('%s-%d' % (m.name, i), m.len, m.fps)
        for i, d in enumerate(DIRS):
            if d.mirror is not None:
                pony.add_mirror_anim('%s-%d' % (m.name, i), dct[d.mirror])


    for sex, ms in (('f', 'mare'), ('m', 'stallion')):
        # Define parts
        base_part = pony.add_part('%s/base' % sex)
        mane_part = pony.add_part('%s/mane' % sex)
        tail_part = pony.add_part('%s/tail' % sex)
        eyes_part = pony.add_part('%s/eyes' % sex)
        equip0_part = pony.add_part('%s/equip0' % sex, optional=True)
        equip1_part = pony.add_part('%s/equip1' % sex, optional=True)
        equip2_part = pony.add_part('%s/equip2' % sex, optional=True)
        dummy_part = pony.add_part('%s/_dummy' % sex)

        # Base layer
        base_variants = {tribe: base_part.add_variant(tribe)
                for tribe in BASES}

        tribe_sheet_dct = {i: make_tribe_sheets(
            lambda l: load('base/%s/%s-%d-%s.png' % (ms, ms, i, l)))
            for i in range(5)}
        for m, i, anim_name in standard_anims():
            for tribe, sheet in tribe_sheet_dct[i].items():
                row = sheet.extract((m.base_col, m.row), size=(m.len, 1))
                anim = row.sheet_to_anim((1, 1), m.fps)
                base_variants[tribe].add_graphics(anim_name, anim)

        # Mane/tail layers
        for kind, idx in standard_manes_tails():
            part = mane_part if kind == 'mane' else tail_part
            variant = part.add_variant('%d' % idx)

            sheet = load('parts/%s/%s%d.png' % (ms, kind, idx))
            sheet = set_depth(sheet, 120)
            for m, i, anim_name in standard_anims():
                x = (m.base_col * 5 + i) * m.len
                y = m.row
                row = sheet.extract((x, y), size=(m.len, 1))
                anim = row.sheet_to_anim((1, 1), m.fps)
                variant.add_graphics(anim_name, anim)

        # Hat box
        variant = dummy_part.add_variant('hat_box')

        hat_box_dct = {i: load('base/%s/%s-%d-hat-box.png' % (ms, ms, i))
                for i in range(5)}
        for m, i, anim_name in standard_anims():
            row = hat_box_dct[i].extract((m.base_col, m.row), size=(m.len, 1))
            anim = row.sheet_to_anim((1, 1), m.fps)
            variant.add_graphics(anim_name, anim)

        # Eyes
        sheet = load1('parts/%s/eyes1.png' % ms)
        sheet = set_depth(sheet, 110)
        add_hat_layer(eyes_part, 'eyes1', sex, sheet)

        # Hats
        for hat_type in ('witch', 'santa', 'party'):
            sheet = load1('equipment/%s-hat-%s.png' % (hat_type, sex))
            sheet = set_depth(sheet, 130)
            add_hat_layer(equip0_part, 'hat/%s' % hat_type, sex, sheet)



