from outpost_data.core.builder2.base import *
from outpost_data.core.sprite import \
        SpriteDef, AnimDef, MirroredAnimDef, LayerDef, DerivedLayerDef


class SpritePrototype(PrototypeBase):
    KIND = 'sprite'
    FIELDS = ('anims', 'derived_anims', 'layers', 'derived_layers', 'graphics')

    def __init__(self, *args, **kwargs):
        super(SpritePrototype, self).__init__(*args, **kwargs)

        self.anims = {}
        self.derived_anims = {}
        self.layers = {}
        self.derived_layers = {}
        self.graphics = {}

    def instantiate(self):
        self.name = self.require('name') or '_%x' % id(self)

        return SpriteDef(self.name,
                self.anims, self.derived_anims,
                self.layers, self.derived_layers,
                self.graphics)

class SpriteBuilder(BuilderBase):
    PROTO_CLASS = SpritePrototype

    def get(self, name):
        b = self.child()
        b._dct[name] = self._dct[name]

    def anim(self, name, length, framerate):
        def f(x, arg):
            x.anims[name] = AnimDef(name, length, framerate)
        self._modify(f, None)

    def mirror_anim(self, name, base_name):
        def f(x, arg):
            x.derived_anims[name] = MirroredAnimDef(name, base_name)
        self._modify(f, None)

    def layer(self, name, size):
        def f(x, arg):
            x.layers[name] = LayerDef(name, size)
        self._modify(f, None)

    def derived_layer(self, name, base_name, func):
        def f(x, arg):
            x.derived_layers[name] = DerivedLayerDef(name, base_name, func)
        self._modify(f, None)

    def add_graphics(self, layer_name, anim_name, anim):
        def f(x, arg):
            x.graphics[(layer_name, anim_name)] = anim
        self._modify(f, None)
