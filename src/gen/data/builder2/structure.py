from outpost_data.core import image2, geom, util
from outpost_data.core.builder2.base import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import Anim
from outpost_data.core.structure import solid, StructureDef2, Model2


DEFAULT_SHAPE = solid(1, 1, 1)

class StructurePrototype(PrototypeBase):
    KIND = 'structure'
    FIELDS = (
            'image', 'mesh', 'image_bounds', 'shape', 'layer', 'parts',
            'light_offset', 'light_color', 'light_radius',
            )

    def clone(self):
        obj = super(StructurePrototype, self).clone()
        obj.parts = self.parts.copy() if self.parts is not None else None
        return obj

    def instantiate(self):
        self.name = self.require('name') or '_%x' % id(self)

        shape = self.require('shape', DEFAULT_SHAPE)
        layer = self.require('layer', 0)

        if self.require_one('mesh', 'parts'):
            mesh = self.mesh
            img = self.require('image', reason='mesh')
            bounds = self.image_bounds or \
                    ((0, 0, 0), tuple(x * TILE_SIZE for x in shape.size))
            parts = [(Model2(mesh, bounds), img)]
        else:
            parts = self.parts
            self.require_unset('image', 'parts')

        s = StructureDef2(self.name, shape, layer)
        for m, i in parts:
            s.add_part(m, i)

        pos, color, radius = self.check_group(
                ('light_offset', 'light_color', 'light_radius'))
        if pos is not None:
            s.set_light(pos or (0, 0, 0), color or (0, 0, 0), radius or 1)

        return s

    def get_image(self):
        """Obtain a reasonable depiction of this structure as a still image."""
        if self.image is not None:
            return self.image.still()
        elif self.shape is not None:
            sx, sy, sz = self.shape.size
            if len(self.parts) == 0:
                return image2.Image(size=(sx, sy + sz), unit=TILE_SIZE)
            size = geom.mul((sx, sy + sz), TILE_SIZE)

            layers = []
            for model, img in self.parts:
                b_min, b_max = model.bounds
                bx, by = util.project(b_min)
                layers.append(img.still().pad(size, offset=(bx, by)))

            return layers[0].stack(layers)
        else:
            return image2.Image(size=(1, 1), unit=TILE_SIZE)


class StructureBuilder(BuilderBase):
    PROTO_CLASS = StructurePrototype

    image = dict_modifier('image')
    mesh = dict_modifier('mesh')
    bounds = dict_modifier('bounds')
    shape = dict_modifier('shape')
    layer = dict_modifier('layer')
    parts = dict_modifier('parts')

    light_offset = dict_modifier('light_offset')
    light_color = dict_modifier('light_color')
    light_radius = dict_modifier('light_radius')

    def light(self, offset, color, radius):
        def f(x, arg):
            x.light_offset = offset
            x.light_color = color
            x.light_radius = radius
        return self._modify(f, None)

    def anim(self, frames, framerate, oneshot=False):
        def f(x, arg):
            x.image = Anim(frames, framerate, oneshot)
        return self._modify(f, None)

    def part(self, *args):
        if len(args) == 1:
            args, = args

        def f(x, part):
            if x.parts is None:
                x.parts = [part]
            else:
                x.parts.append(part)
        return self._dict_modify(f, args)

    def mesh_part(self, mesh, img, bounds=None):
        def f(x, arg):
            bounds_ = bounds
            if bounds_ is None:
                bounds_ = ((0, 0, 0), tuple(y * TILE_SIZE for y in x.shape.size))
            part = (Model2(mesh, bounds_), img)
            if x.parts is None:
                x.parts = [part]
            else:
                x.parts.append(part)
        return self._dict_modify(f, None)
