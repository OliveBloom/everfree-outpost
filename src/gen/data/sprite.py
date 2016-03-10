from PIL import Image

from outpost_data.core import util
from outpost_data.core.consts import *
from outpost_data.core.image2 import Anim


# Size of a sheet in frames.
# Note that 16 * 96 = 1536, which is a reasonable pixel size.
ANIM_SHEET_SIZE = (16, 16)


class SpriteDef:
    def __init__(self, name, anims, derived_anims, layers, derived_layers, graphics):
        self.name = name
        self.anims = anims
        self.derived_anims = derived_anims
        self.layers = layers
        self.derived_layers = derived_layers
        self.graphics = graphics

        self.num_sheets = None

        self.id = None

    def base_anim(self, name, reason):
        val = self.anims.get(name)
        if val is None:
            if name in self.derived_anims:
                raise KeyError(
                        ('animation %s/%s refers to derived base %s/%s ' +
                            '(it should refer to a non-derived base instead)') %
                        (self.name, reason, self.name, name))
            else:
                raise KeyError(
                        'animation %s/%s refers to nonexistent base %s/%s' %
                        (self.name, reason, self.name, name))
        return val

    def base_layer(self, name, reason):
        val = self.layers.get(name)
        if val is None:
            if name in self.derived_layers:
                raise KeyError(
                        ('layer %s/%s refers to derived base %s/%s ' +
                            '(it should refer to a non-derived base instead)') %
                        (self.name, reason, self.name, name))
            else:
                raise KeyError(
                        'layer %s/%s refers to nonexistent base %s/%s' %
                        (self.name, reason, self.name, name))
        return val

    def anim(self, name):
        val = self.anims.get(name)
        if val is None:
            val = self.derived_anims.get(name)
            if val is None:
                raise KeyError('no such animation: %s/%s' % (self.name, name))
        return val

    def layer(self, name):
        val = self.layers.get(name)
        if val is None:
            val = self.derived_layers.get(name)
            if val is None:
                raise KeyError('no such layer: %s/%s' % (self.name, name))
        return val

    def all_anims(self):
        for a in self.anims.values():
            yield a
        for a in self.derived_anims.values():
            yield a

    def all_layers(self):
        for l in self.layers.values():
            yield l
        for l in self.derived_layers.values():
            yield l


class AnimDef:
    # TODO: oneshot?
    def __init__(self, name, length, framerate):
        self.name = name
        self.length = length
        self.framerate = framerate
        self.mirror = False

        self.sheet = None
        self.offset = None

        self.full_name = None
        self.id = None

class MirroredAnimDef:
    def __init__(self, name, base_name):
        self.name = name
        self.base_name = base_name
        self.mirror = True

        self.length = None
        self.framerate = None

        self.sheet = None
        self.offset = None

        self.full_name = None
        self.id = None

class LayerDef:
    def __init__(self, name, size):
        self.name = name
        self.size = size

        self.graphics = {}

        self.full_name = None

class DerivedLayerDef:
    def __init__(self, name, base_name, func):
        self.name = name
        self.base_name = base_name
        self.func = func

        self.size = None
        self.graphics = {}

        self.full_name = None


class AnimWrapper:
    '''Wrapper class to present the full name to assign_ids.'''
    def __init__(self, s, a):
        self.name = '%s/%s' % (s.name, a.name)
        self.a = a

    @property
    def id(self):
        return self.a.id

    @id.setter
    def id(self, value):
        self.a.id = value

def assign_anim_ids(sprites):
    # Sanity check
    for s in sprites:
        for a in s.derived_anims:
            assert a not in s.anims, \
                    'name collision between derived and non-derived animations %s/%s' % \
                    (s.name, a.name)

        for l in s.derived_layers:
            assert l not in s.layers, \
                    'name collision between derived and non-derived layers %s/%s' % \
                    (s.name, l.name)

    # Assign global IDs to all anims
    dct = util.assign_ids([AnimWrapper(s, a)
        for s in sprites for a in s.all_anims()])

    # Set full name for all anims and layers
    for s in sprites:
        for a in s.all_anims():
            a.full_name = '%s/%s' % (s.name, a.name)

        for l in s.all_layers():
            l.full_name = '%s/%s' % (s.name, l.name)

    return dct


def process_anims(sprites):
    for s in sprites:
        # Place all ordinary animations
        anims = sorted(s.anims.values(), key=lambda a: a.id)
        boxes = [(a.length, 1) for a in anims]
        num_sheets, offsets = util.pack_boxes(ANIM_SHEET_SIZE, boxes)
        assert num_sheets == 1, \
                'anims for %r overflowed onto multiple sheets (unsupported)' % s.name

        for a, (sheet_idx, offset) in zip(anims, offsets):
            a.sheet = sheet_idx
            a.offset = offset

        # Copy data into derived animations
        for a in s.derived_anims.values():
            base_a = s.base_anim(a.base_name, reason=a.name)

            a.length = base_a.length
            a.framerate = base_a.framerate
            a.sheet = base_a.sheet
            a.offset = base_a.offset

    return sorted((a for s in sprites for a in s.all_anims()), key=lambda a: a.id)

def build_sheets_one(sprite):
    # Distribute graphics to layers
    for (l_name, a_name), img in sprite.graphics.items():
        # NB: It *is* legal to define graphics for derived layers.  Graphics
        # defined this way will override the auto-generated version.
        assert l_name in sprite.layers or l_name in sprite.derived_layers, \
                'sprite %r contains graphics for nonexistent layer %r' % \
                (sprite.name, l_name)
        assert a_name in sprite.anims, \
                'sprite %r contains graphics for %s animation %r' % \
                (sprite.name,
                 'nonexistent' if a_name not in sprite.derived_anims else 'derived',
                 a_name)

        sprite.layers[l_name].graphics[a_name] = img

    # Generate additional graphics for derived layers
    for l in sprite.derived_layers.values():
        base_l = sprite.base_layer(l.base_name, reason=l.name)

        for k,v in base_l.graphics.items():
            if k not in l.graphics:
                frames = [l.func(k, v.get_frame(i)) for i in range(v.length)]
                l.graphics[k] = Anim(frames, v.rate, v.oneshot)

        l.size = base_l.size

    # Place graphics into sheets
    sheets = {}
    for l in sprite.all_layers():
        lw, lh = l.size
        def mul(v):
            x, y = v
            return (lw * x, lh * y)
        # TODO: probably a good idea to condense sheets (in general) to avoid
        # wasting VRAM on lots of unused transparent pixels
        img = Image.new('RGBA', mul(ANIM_SHEET_SIZE))

        for a in sprite.anims.values():
            if a.name not in l.graphics:
                continue
            g = l.graphics[a.name].flatten().raw().raw()
            img.paste(g, mul(a.offset))

        sheet_name = '%s/%s' % (sprite.name, l.name)
        sheets[sheet_name.replace('/', '_')] = img

    return sheets

def build_sheets(sprites):
    sheets = {}
    for s in sprites:
        dct = build_sheets_one(s)

        for k,v in dct.items():
            assert k not in sheets, \
                    'duplicate sheet name %r (second instance was in sprite %r)' % (k, s.name)
            sheets[k] = v

    return sheets


def build_anim_client_json(anims):
    def convert(a):
        return {
                'length': a.length,
                'framerate': a.framerate,
                'mirror': a.mirror,
                'sheet': a.sheet,
                'offset': a.offset,
                }
    return list(convert(a) for a in anims)

def build_anim_server_json(anims):
    def convert(a):
        return {
                'name': a.full_name,
                'length': a.length,
                'framerate': a.framerate,
                }
    return list(convert(a) for a in anims)
