from outpost_data.core import sprite, image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import loader, Image, Anim
from outpost_data.outpost.lib.pony_sprite import *
from outpost_data.outpost.lib.sprite_util import *


def base():
    pony = SPRITE.new('pony', SPRITE_SIZE)
    load = loader('sprites', unit=SPRITE_SIZE)
    load1 = loader('sprites')

    # Define animations
    for m in MOTIONS:
        for i, d in enumerate(DIRS):
            if d.mirror is None:
                pony.add_anim('%s-%d' % (m.name, i), m.len, m.fps)
            else:
                pony.derive_anim('%s-%d' % (m.name, i),
                        '%s-%d' % (m.name, d.mirror), sprite.mirror_anim)

    for sex, ms in (('f', 'mare'), ('m', 'stallion')):
        # Base layers
        for layer in ('base', 'horn', 'frontwing', 'backwing'):
            layer_name = '%s/%s' % (sex, layer)
            pony.add_layer(layer_name)

            for m, i, anim_name in standard_anims():
                sheet = load('base/%s/%s-%d-%s.png' % (ms, ms, i, layer))
                row = sheet.extract((m.base_col, m.row), size=(m.len, 1))
                anim = row.sheet_to_anim((1, 1), m.fps)
                pony.add_graphics(layer_name, anim_name, anim)

        # Mane/tail layers
        for kind, idx in standard_manes_tails():
            layer_name = '%s/%s/%d' % (sex, kind, idx)
            pony.add_layer(layer_name)

            sheet = load('parts/%s/%s%d.png' % (ms, kind, idx))
            for m, i, anim_name in standard_anims():
                x = (m.base_col * 5 + i) * m.len
                y = m.row
                row = sheet.extract((x, y), size=(m.len, 1))
                anim = row.sheet_to_anim((1, 1), m.fps)
                pony.add_graphics(layer_name, anim_name, anim)

        # Hat box
        layer_name = '%s/_dummy/hat_box' % sex
        pony.add_layer(layer_name)

        for m, i, anim_name in standard_anims():
            sheet = load('base/%s/%s-%d-hat-box.png' % (ms, ms, i))
            row = sheet.extract((m.base_col, m.row), size=(m.len, 1))
            anim = row.sheet_to_anim((1, 1), m.fps)
            pony.add_graphics(layer_name, anim_name, anim)

        # Eyes
        sheet = load1('parts/%s/eyes1.png' % ms)
        add_hat_layer('%s/eyes1' % sex, sex, sheet)

        # Hats
        for hat_type in ('witch', 'santa', 'party'):
            sheet = load1('equipment/%s-hat-%s.png' % (hat_type, sex))
            add_hat_layer('%s/hat/%s' % (sex, hat_type), sex, sheet)


def sleep():
    pony = get_pony_sprite()
    load = loader('sprites', unit=SPRITE_SIZE)
    load1 = loader('sprites')

    # Define animations
    pony.add_anim('sleep-0', 6, 2)
    pony.derive_anim('sleep-4', 'sleep-0', sprite.mirror_anim)

    def make_anim(row):
        frames = [
                row.extract((0, 0)),
                row.extract((0, 0)),
                row.extract((1, 0)),
                row.extract((2, 0)),
                row.extract((2, 0)),
                row.extract((1, 0)),
                ]
        return Anim(frames, 2)

    for sex, ms in (('f', 'mare'), ('m', 'stallion')):
        # Base layers + hat box
        sheet = load('base/sleep-%s.png' % (sex,))
        for i, layer in enumerate(('base', 'horn', 'frontwing', 'backwing', '_dummy/hat_box')):
            row = sheet.extract((0, i), size=(3, 1))
            anim = make_anim(row)
            pony.add_graphics('%s/%s' % (sex, layer), 'sleep-0', anim)

        # Mane/tail layers
        for kind, idx in standard_manes_tails():
            sheet = load('parts/%s/%s%d.png' % (ms, kind, idx))
            row = sheet.extract((10, 0), size=(3, 1))
            anim = make_anim(row)
            pony.add_graphics('%s/%s/%d' % (sex, kind, idx), 'sleep-0', anim)

        # Eyes
        blank = Image(size=(3, 1), unit=SPRITE_SIZE)
        anim = make_anim(blank)
        pony.add_graphics('%s/eyes1' % sex, 'sleep-0', anim)

        continue
        # Hat box
        layer_name = '%s/_dummy/hat_box' % sex
        pony.add_layer(layer_name)

        for m, i, anim_name in standard_anims():
            sheet = load('base/%s/%s-%d-hat-box.png' % (ms, ms, i))
            row = sheet.extract((m.base_col, m.row), size=(m.len, 1))
            anim = row.sheet_to_anim((1, 1), m.fps)
            pony.add_graphics(layer_name, anim_name, anim)

        # Eyes
        sheet = load1('parts/%s/eyes1.png' % ms)
        add_hat_layer('%s/eyes1' % sex, sex, sheet)

        # Hats
        for hat_type in ('witch', 'santa', 'party'):
            sheet = load1('equipment/%s-hat-%s.png' % (hat_type, sex))
            add_hat_layer('%s/hat/%s' % (sex, hat_type), sex, sheet)

def init():
    base()
    sleep()


