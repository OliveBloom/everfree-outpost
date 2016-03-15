from PIL import Image

from outpost_data.core import util
from outpost_data.core.consts import *
from outpost_data.core.image2 import Anim


# Size of a sheet in frames.
# Note that 16 * 96 = 1536, which is a reasonable pixel size.
ANIM_SHEET_SIZE = (16, 16)




class AnimDef:
    def __init__(self, owner, name, length, rate):
        self.owner = owner
        self.name = name
        self.length = length
        self.rate = rate
        self.mirror = False

        self.sheet = None
        self.offset = None

        self.id = None

    @property
    def full_name(self):
        return '%s//%s' % (self.owner.name, self.name)

class DerivedAnimDef:
    def __init__(self, owner, name, base, func):
        self.owner = owner
        self.name = name
        self.base = base
        self.func = func

        self.id = None

    @property
    def full_name(self):
        return '%s//%s' % (self.owner.name, self.name)

class PartDef:
    def __init__(self, owner, name, optional=False):
        self.owner = owner
        self.name = name
        self.optional = optional
        self.variants = {}

        self.id = None

    @property
    def full_name(self):
        return '%s//%s' % (self.owner.name, self.name)


    def _add_variant(self, variant):
        assert variant.name not in self.variants, \
                'duplicate variant %r for part %r' % (variant.name, self.full_name)
        self.variants[variant.name] = variant
        return variant

    def add_variant(self, name):
        return self._add_variant(VariantDef(self, name))

    def add_derived_variant(self, name, base, func):
        assert base.owner.owner is self.owner, \
                'base for derived variant %r must belong to sprite %r (base = %r)' % \
                ('%s//%s' % (self.full_name, name), self.owner.name, base.full_name)
        assert base.func is None, \
                'base for derived variant %r must not be a derived variant (base = %r)' % \
                ('%s//%s' % (self.full_name, name), base.full_name)
        return self._add_variant(VariantDef(self, name, base, func))

    def get_variant(self, name):
        v = self.variants.get(name)
        if v is None:
            raise KeyError('no such variant %r in part %r' % (name, self.full_name))
        return v

    def iter_variants(self):
        return self.variants.values()

class VariantDef:
    def __init__(self, owner, name, base=None, func=None):
        self.owner = owner
        self.name = name
        self.graphics = {}
        assert (base is None) == (func is None)
        self.base = base
        self.func = func

        self.file_name = None

        self.id = None
        self.local_id = None

    @property
    def full_name(self):
        return '%s//%s' % (self.owner.full_name, self.name)

    def add_graphics(self, anim_name, anim):
        anim_def = self.owner.owner.get_anim(anim_name)
        assert anim.length == anim_def.length, \
                'animation for %r must match length of %r animation definition (%d != %d)' % \
                (self.full_name, anim_name, anim.length, anim_def.length)
        assert anim_name not in self.graphics, \
                'duplicate graphics for animation %r of variant %r' % \
                (anim_name, self.full_name)
        self.graphics[anim_name] = anim

    def get_graphics(self, anim_name):
        g = self.graphics.get(anim_name)
        if g is None and self.func is not None:
            sprite = self.owner.owner
            anim = sprite.get_anim(anim_name)
            base_g = self.base.get_graphics(anim_name)
            if base_g is not None:
                frames = [self.func(self, anim, i, base_g.get_frame(i))
                        for i in range(base_g.length)]
                g = Anim(frames, base_g.rate, base_g.oneshot)
                self.graphics[anim_name] = g
        return g

class SpriteDef:
    def __init__(self, name, size):
        self.name = name
        self.size = size
        self.anims = {}
        self.parts = {}

        self.num_sheets = None

        self.id = None


    def _add_anim(self, anim):
        assert anim.name not in self.anims, \
                'duplicate animation %r in sprite %r' % (anim.name, self.name)
        self.anims[anim.name] = anim
        return anim

    def add_anim(self, name, length, rate):
        return self._add_anim(AnimDef(self, name, length, rate))

    def add_mirror_anim(self, name, base):
        return self._add_anim(DerivedAnimDef(self, name, base, gen_mirror_anim))

    def get_anim(self, name):
        a = self.anims.get(name)
        if a is None:
            raise KeyError('no such animation %r in sprite %r' % (name, self.name))
        return a

    def iter_anims(self):
        return self.anims.values()

    def iter_base_anims(self):
        return (a for a in self.iter_anims() if isinstance(a, AnimDef))


    def add_part(self, name, optional=False):
        assert name not in self.parts, \
                'duplicate part %r in sprite %r' % (name, self.name)
        part = PartDef(self, name, optional)
        self.parts[name] = part
        return part

    def get_part(self, name):
        p = self.parts.get(name)
        if p is None:
            raise KeyError('no such part %r in sprite %r' % (name, self.name))
        return p

    def iter_parts(self):
        return self.parts.values()


def gen_mirror_anim(owner, name, id, base):
    a = AnimDef(owner, name, base.length, base.rate)
    a.sheet = base.sheet
    a.offset = base.offset
    a.id = id
    a.mirror = not base.mirror
    return a


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

def process_anims(sprites):
    for s in sprites:
        # Place all ordinary animations
        anims = sorted(s.iter_base_anims(), key=lambda a: a.id)
        boxes = [(a.length, 1) for a in anims]
        num_sheets, offsets = util.pack_boxes(ANIM_SHEET_SIZE, boxes)
        assert num_sheets == 1, \
                'anims for %r overflowed onto multiple sheets (unsupported)' % s.name

        for a, (sheet_idx, offset) in zip(anims, offsets):
            a.sheet = sheet_idx
            a.offset = offset

        # Generate all derived animations
        for k,v in s.anims.items():
            if isinstance(v, DerivedAnimDef):
                new_anim = v.func(s, v.name, v.id, v.base)
                s.anims[k] = new_anim

    # Return sorted list of all animations (used for JSON output)
    return sorted((a for s in sprites for a in s.iter_anims()), key=lambda a: a.id)

def process_parts(sprites):
    return sorted((p for s in sprites for p in s.iter_parts()), key=lambda p: p.id)

def assign_sub_ids(sprites):
    util.assign_ids([FullNameWrapper(a)
        for s in sprites for a in s.iter_anims()])
    util.assign_ids([FullNameWrapper(p)
        for s in sprites for p in s.iter_parts()])
    util.assign_ids([FullNameWrapper(v)
        for s in sprites for p in s.iter_parts() for v in p.iter_variants()])

    for s in sprites:
        for p in s.iter_parts():
            base = 1 if p.optional else 0
            for i, v in enumerate(sorted(p.iter_variants(), key=lambda v: v.name)):
                v.local_id = i + base


def build_variant_sheet(variant):
    sprite = variant.owner.owner
    sw, sh = sprite.size
    def mul(v):
        x, y = v
        return (sw * x, sh * y)

    # TODO: probably a good idea to condense sheets (in general) to avoid
    # wasting VRAM on lots of unused transparent pixels
    img = Image.new('RGBA', mul(ANIM_SHEET_SIZE))

    for anim in sprite.iter_anims():
        g = variant.get_graphics(anim.name)
        if g is None:
            continue
        img.paste(g.flatten().raw().raw(), mul(anim.offset))

    return img

def build_sprite_sheets(sprite):
    sheets = []
    for p in sprite.iter_parts():
        for v in p.iter_variants():
            sheets.append((v, build_variant_sheet(v)))
    return sheets

def build_sheets(sprites):
    sheets = []
    for s in sprites:
        sheets.extend(build_sprite_sheets(s))
    return sheets


def build_anim_client_json(anims):
    def convert(a):
        return {
                'length': a.length,
                'framerate': a.rate,
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
                'framerate': a.rate,
                }
    return list(convert(a) for a in anims)

def build_part_client_json(parts):
    def convert(p):
        variants = [None] * (len(p.variants) + (1 if p.optional else 0))
        for v in p.iter_variants():
            variants[v.local_id] = v.id
        return {
                'variants': variants
                }
    return list(convert(p) for p in parts)
