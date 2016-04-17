from collections import namedtuple
import PIL

from outpost_data.core import util
from outpost_data.core.consts import *
from outpost_data.core.image2 import Anim, Image


# Size of a sheet in frames.
# Note that 16 * 96 = 1536, which is a reasonable pixel size.
ANIM_SHEET_SIZE = (16, 16)


class AnimDef:
    def __init__(self, owner, name, length, rate, base=None, func=None):
        self.owner = owner
        self.name = name
        self.length = length
        self.rate = rate

        self.base = base
        self.func = func

        self.id = None
        self.local_id = None

    @property
    def full_name(self):
        return '%s//%s' % (self.owner.name, self.name)

class LayerDef:
    def __init__(self, owner, name, base=None, func=None):
        self.owner = owner
        self.name = name

        self.base = base
        self.func = func

        self.gfx_start = None
        self.gfx_count = None

        self.id = None

    @property
    def full_name(self):
        return '%s//%s' % (self.owner.name, self.name)

class GraphicsDef:
    def __init__(self, anim):
        self.anim = anim

        self.base = None

        self.sheet = None
        self.src_offset = None
        self.dest_offset = None
        self.size = None
        self.mirror = False

        self.id = None

class SpriteDef:
    def __init__(self, name, size):
        self.name = name
        self.size = size

        self.layers = {}
        self.anims = {}
        self.graphics = {}

    def add_anim(self, name, length, rate):
        assert name not in self.anims
        self.anims[name] = AnimDef(self, name, length, rate)

    def derive_anim(self, name, base, func):
        assert name not in self.anims
        self.anims[name] = AnimDef(self, name, None, None, base, func)

    def get_anim(self, name):
        return self.anims[name]


    def add_layer(self, name):
        assert name not in self.layers
        self.layers[name] = LayerDef(self, name)

    def derive_layer(self, name, base, func):
        assert name not in self.layers
        self.layers[name] = LayerDef(self, name, base, func)

    def get_layer(self, name):
        return self.layers[name]


    def add_graphics(self, layer_name, anim_name, anim):
        key = (layer_name, anim_name)
        assert key not in self.graphics
        self.graphics[key] = GraphicsDef(anim)

    def get_graphics(self, layer_name, anim_name):
        key = (layer_name, anim_name)
        if key in self.graphics:
            return self.graphics[key]

        anim = self.anims[anim_name]
        layer = self.layers[layer_name]

        # Try to generate it by deriving from another animation
        if anim.base is not None:
            orig = self.get_graphics(layer_name, anim.base)
            if orig is not None:
                derived = anim.func(layer, anim, orig)
                self.graphics[key] = derived
                return derived

        # Try to generate it by deriving from another layer
        if layer.base is not None:
            orig = self.get_graphics(layer.base, anim_name)
            if orig is not None:
                derived = layer.func(layer, anim, orig)
                self.graphics[key] = derived
                return derived

        return None


    def iter_layers(self):
        return (l for l in self.layers.values() if '_dummy' not in l.name)


def mirror_anim(layer, anim, orig):
    gfx = GraphicsDef(None)
    gfx.base = orig
    gfx.mirror = not gfx.base.mirror
    return gfx


class FullNameWrapper:
    '''Wrapper class to present the full name to assign_ids.'''
    def __init__(self, obj):
        self.obj = obj

    @property
    def name(self):
        return self.obj.full_name

    @property
    def id(self):
        return self.obj.id

    @id.setter
    def id(self, value):
        self.obj.id = value


def process(sprites):
    util.assign_ids([FullNameWrapper(a)
        for s in sprites for a in s.anims.values()])
    util.assign_ids([FullNameWrapper(l)
        for s in sprites for l in s.iter_layers()])

    gfx_index = 0
    for s in sprites:
        sorted_anims = sorted(s.anims.values(), key=lambda a: a.id)
        sorted_layers = sorted(s.iter_layers(), key=lambda l: l.id)

        for i,a in enumerate(sorted_anims):
            a.local_id = i

            if a.base is not None:
                base = s.anims[a.base]
                a.length = base.length
                a.rate = base.rate

        # Fill out the remaining slots of s.graphics
        for l in sorted_layers:
            for i,a in enumerate(sorted_anims):
                g = s.get_graphics(l.name, a.name)
                if g is None:
                    util.warn('sprite %r: layer %r has no graphics for anim %r' %
                            (s.name, l.name, a.name))
                    continue
                g.id = gfx_index + i
            l.gfx_start = gfx_index
            l.gfx_count = len(s.anims)
            gfx_index += len(s.anims)

        # Autocrop anims
        for g in s.graphics.values():
            if g.base is None:
                g.anim, g.dest_offset = g.anim.autocrop()
                g.size = g.anim.px_size

def collect_defs(sprites):
    anims = sorted((a for s in sprites for a in s.anims.values()), key=lambda x: x.id)
    layers = sorted((l for s in sprites for l in s.iter_layers()), key=lambda x: x.id)

    num_graphics = max(l.gfx_start + l.gfx_count for l in layers)
    graphics = [None] * num_graphics
    for s in sprites:
        for g in s.graphics.values():
            if g.id is not None:
                graphics[g.id] = g

    return (anims, layers, graphics)

def build_sheets(sprites):
    # NB: gfxs only includes non-derived graphics
    gfxs = sorted((g for s in sprites for g in s.graphics.values()
        if g.base is None and g.id is not None),
        key=lambda g: g.id)
    def size(g):
        w, h = g.anim.flatten().px_size
        return ((w + 15) // 16, (h + 15) // 16)
    boxes = [size(g) for g in gfxs]
    num_sheets, offsets = util.pack_boxes((2048 // 16, 2048 // 16), boxes)

    for g, (sheet, (off_x, off_y)) in zip(gfxs, offsets):
        g.sheet = sheet
        g.src_offset = (off_x * 16, off_y * 16)

    # Update derived graphics
    for s in sprites:
        for g in s.graphics.values():
            if g.base is not None:
                g.sheet = g.base.sheet
                g.src_offset = g.base.src_offset
                g.dest_offset = g.base.dest_offset
                g.size = g.base.size

    # Generate sheets
    return [
            Image.sheet([(g.anim.flatten(), g.src_offset)
                for g in gfxs if g.sheet == i],
                (2048, 2048))
            for i in range(num_sheets)
            ]


def build_anim_client_json(anims):
    def convert(a):
        return {
                'length': a.length,
                'framerate': a.rate,
                'local_id': a.local_id,
                }
    return list(convert(a) for a in anims)

def build_anim_server_json(anims):
    def convert(a):
        return {
                'name': a.full_name,
                'length': a.length,
                'framerate': a.rate,
                }
    return list(convert(a) for a in anims)

def build_layer_client_json(layers):
    def convert(l):
        return {
                'start': l.gfx_start,
                'count': l.gfx_count,
                }
    return list(convert(l) for l in layers)

def build_layer_server_json(layers):
    def convert(l):
        return {
                'name': l.full_name,
                }
    return list(convert(l) for l in layers)

def build_graphics_client_json(graphics):
    def convert(g):
        if g is None:
            return None
        return {
                'sheet': g.sheet,
                'src_offset': g.src_offset,
                'dest_offset': g.dest_offset,
                'size': g.size,
                'mirror': g.mirror,
                }
    return list(convert(g) for g in graphics)
