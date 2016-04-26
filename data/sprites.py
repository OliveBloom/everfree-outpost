from outpost_data.core import sprite, image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import loader, Image, Anim
from outpost_data.outpost.lib.pony_sprite import *
from outpost_data.outpost.lib.sprite_util import *


def extract_graphics(sheets, m, i, anim_name, row_to_anim):
    pony = get_pony_sprite()

    for layer_name, sheet in sheets.items():
        row = sheet.extract((m.base_col + i * m.len, m.row), size=(m.len, 1))
        anim = row_to_anim(row, m)
        pony.add_graphics(layer_name, anim_name, anim)


def init():
    pony = SPRITE.new('pony', SPRITE_SIZE)
    load = loader('sprites', unit=SPRITE_SIZE)
    load1 = loader('sprites')


    # Standard animations

    # Define animations
    for m in MOTIONS:
        for i, d in enumerate(DIRS):
            if d.mirror is None:
                pony.add_anim('%s-%d' % (m.name, i), m.len, m.fps)
            else:
                pony.derive_anim('%s-%d' % (m.name, i),
                        '%s-%d' % (m.name, d.mirror), sprite.mirror_anim)

    # Define layers and collect sheets
    sheets = {}
    for sex, ms in (('f', 'mare'), ('m', 'stallion')):
        # Base layers
        for layer in ('base', 'horn', 'frontwing', 'backwing'):
            layer_name = '%s/%s' % (sex, layer)
            pony.add_layer(layer_name)
            sheets[layer_name] = load('base/%s/%s.png' % (ms, layer))

        # Mane/tail layers
        for kind, idx in standard_manes_tails():
            layer_name = '%s/%s/%d' % (sex, kind, idx)
            pony.add_layer(layer_name)
            sheets[layer_name] = load('parts/%s/%s%d.png' % (ms, kind, idx))

        # Hat box
        layer_name = '%s/_dummy/hat_box' % sex
        pony.add_layer(layer_name)
        sheets[layer_name] = load('base/%s/hat-box.png' % ms)

    # Add graphics
    row_to_anim = lambda row, m: row.sheet_to_anim((1, 1), m.fps)
    for m, i, anim_name in standard_anims():
        extract_graphics(sheets, m, i, anim_name, row_to_anim)


    # Other anims

    # Sleep

    pony.add_anim('sleep-0', 6, 2)
    pony.derive_anim('sleep-4', 'sleep-0', sprite.mirror_anim)
    def make_sleep_anim(row, _m):
        frames = [
                row.extract((0, 0)),
                row.extract((0, 0)),
                row.extract((1, 0)),
                row.extract((2, 0)),
                row.extract((2, 0)),
                row.extract((1, 0)),
                ]
        return Anim(frames, 2)
    extract_graphics(sheets, Motion('sleep', 0, 10, 3, 2), 0, 'sleep-0', make_sleep_anim)

    for sex in ('m', 'f'):
        blank = Image(size=(3, 1), unit=SPRITE_SIZE)
        anim = make_sleep_anim(blank, None)
        pony.add_graphics('%s/eyes1' % sex, 'sleep-0', anim)


    # Hat layers

    for sex, ms in (('f', 'mare'), ('m', 'stallion')):
        # Eyes
        sheet = load1('parts/%s/eyes1.png' % ms)
        add_hat_layer('%s/eyes1' % sex, sex, sheet)

        # Hats
        for hat_type in ('witch', 'santa', 'party'):
            sheet = load1('equipment/%s-hat-%s.png' % (hat_type, sex))
            add_hat_layer('%s/hat/%s' % (sex, hat_type), sex, sheet)
