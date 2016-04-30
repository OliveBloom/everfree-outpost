import base64
import hashlib
import json

import PIL

from outpost_data.core import sprite, image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import loader, Image, Anim
from outpost_data.outpost.lib.pony_sprite import *
from outpost_data.outpost.lib.sprite_util import *

from equip_sprites_render import Renderer

class Context:
    def __init__(self, renderer, json_str, desc):
        self.r = renderer
        self.j = json.loads(json_str)
        self.desc = (hashlib.sha1(json_str.encode()).digest(), desc)

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

def add_equip_layer(layer_name, base_name, json_path):
    r = Renderer()
    with open(json_path) as f:
        json_str = f.read()
    ctx = Context(r, json_str, ())

    pony = get_pony_sprite()
    pony.derive_layer(layer_name, base_name,
            mk_derive_graphics(ctx, '%s//%s' % (pony.name, base_name)))

def init():
    # TODO: don't hardcode . as $root!
    path = 'assets/sprites/equipment/uvdata-sock-f.json'

    add_equip_layer('f/sock/solid/red', 'f/base', path)
    add_equip_layer('m/sock/solid/red', 'm/base', path)
