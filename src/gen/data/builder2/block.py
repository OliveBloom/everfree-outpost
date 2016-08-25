from PIL import Image

from outpost_data.core.builder2.base import *
from outpost_data.core.consts import *
from outpost_data.core.block import BlockDef


def flags_from_shape(s):
    if s == 'empty':
        return B_OCCUPIED
    elif s == 'floor':
        return B_OCCUPIED | B_SUBFLOOR
    else:
        return B_OCCUPIED | B_SOLID_SHAPE(SHAPE_ID[s])

class BlockPrototype(PrototypeBase):
    KIND = 'block'
    FIELDS = ('flags', 'top', 'bottom', 'front', 'back')

    def instantiate(self):
        name = self.require('name') or '_%x' % id(self)
        flags = self.require('flags') or flags_from_shape('solid')
        tiles = {}
        for side in BLOCK_SIDES:
            x = getattr(self, side)
            if x is not None:
                tiles[side] = raw_image(x)

        return BlockDef(name, flags, tiles)

class BlockBuilder(BuilderBase):
    PROTO_CLASS = BlockPrototype

    flags = dict_modifier('flags')
    top = dict_modifier('top')
    bottom = dict_modifier('bottom')
    front = dict_modifier('front')
    back = dict_modifier('back')

    def shape(self, s):
        if isinstance(s, str):
            return self.flags(flags_from_shape(s))
        else:
            return self.flags({k: flags_from_shape(v) for k,v in s.items()})
