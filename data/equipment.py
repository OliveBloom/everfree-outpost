import base64
import hashlib
import json

import PIL.Image
import PIL.ImageChops

from outpost_data.core import sprite, image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import load, Image, Anim
from outpost_data.outpost.lib.pony_sprite import *
from outpost_data.outpost.lib.sprite_util import *

import equip_sprites_render
from equip_sprites_render import Renderer


with open(equip_sprites_render.__file__, 'rb') as f:
    h = hashlib.sha1()
    while True:
        b = f.read()
        if len(b) == 0:
            break
        h.update(b)
    RENDERER_HASH = h.digest()


class Context:
    def __init__(self, renderer, json_str, desc):
        self.r = renderer
        self.j = json.loads(json_str)
        self.desc = (desc, RENDERER_HASH,
                hashlib.sha1(json_str.encode()).digest())

def render_frame(ctx, base_img, frame_name):
    def f(raw_img):
        ctx.r.set_base(raw_img.size[0], raw_img.size[1], raw_img.tobytes())

        for i in (3, 2, 1, 0):
            key = '%s$%d' % (frame_name, i)
            if key not in ctx.j:
                continue
            mask = base64.b64decode(ctx.j[key])
            ctx.r.render_part(mask)

        return PIL.Image.frombytes('RGBA', raw_img.size, ctx.r.get_image())

    return base_img.modify(f, desc=(ctx.desc, frame_name))

def render_anim(ctx, base_anim, layer_name, anim_name):
    frames = [
            render_frame(ctx, img, '%s$%s$%d' % (layer_name, anim_name, i))
            for i, img in enumerate(base_anim._frames)
            ]
    return Anim(frames, base_anim.rate, oneshot=base_anim.oneshot)

def mk_derive_graphics(ctx, base_layer_name):
    def derive_graphics(layer, anim_def, orig):
        new_anim = render_anim(ctx, orig.anim, base_layer_name, anim_def.full_name)
        return GraphicsDef(new_anim)
    return derive_graphics

def add_equip_layer(layer_name, base_name, json_path, style, style_args):
    r = Renderer()
    getattr(r, 'set_style_%s' % style)(*style_args)
    with open(json_path) as f:
        json_str = f.read()
    ctx = Context(r, json_str, (style, style_args))

    pony = get_pony_sprite()
    pony.derive_layer(layer_name, base_name,
            mk_derive_graphics(ctx, '%s//%s' % (pony.name, base_name)))

COLORS = [
        ('red',     (0xcc, 0x44, 0x44)),
        ('orange',  (0xcc, 0x88, 0x44)),
        ('yellow',  (0xee, 0xee, 0x66)),
        ('green',   (0x44, 0xcc, 0x44)),
        ('blue',    (0x44, 0x44, 0xcc)),
        ('purple',  (0xcc, 0x44, 0xcc)),
        ('white',   (0xee, 0xee, 0xee)),
        ('black',   (0x44, 0x44, 0x44)),
        ]

def multiply_image(img, color):
    def f(raw):
        overlay = PIL.Image.new('RGBA', raw.size)
        overlay.paste(color)
        return PIL.ImageChops.multiply(raw, overlay)
    return img.modify(f, desc=color)

def init():
    icon = load('icons/socks.png')

    # TODO: don't hardcode . as $root!
    # TODO: also make sure this file gets listed in data.d
    path = 'assets/sprites/equipment/uvdata-socks.json'

    for name, rgb in COLORS:
        for sex in ('m', 'f'):
            add_equip_layer('%s/socks/solid/%s' % (sex, name), '%s/base' % sex,
                    path, 'solid', rgb)

        ITEM.new('socks/solid/%s' % name) \
                .display_name('%s%s Socks' % (name[0].upper(), name[1:])) \
                .icon(multiply_image(icon, rgb + (255,)))

    ITEM.new('ability/remove_socks') \
            .display_name('Remove Socks') \
            .icon(load('icons/remove-socks.png'))

