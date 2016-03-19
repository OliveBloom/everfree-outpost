from outpost_data.core import image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import loader, Anim
from outpost_data.outpost.lib.pony_sprite import *
from outpost_data.outpost.lib.sprite_util import *


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



