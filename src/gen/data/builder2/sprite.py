from outpost_data.core.builder2.base import *
from outpost_data.core.sprite import SpriteDef

# Not really a proper builder, just a container for a bunch of sprites.
class SpriteBuilder:
    def __init__(self):
        self._dct = {}

    def new(self, name, size):
        assert name not in self._dct, \
                'duplicate sprite with name %r' % (name,)
        s = SpriteDef(name, size)
        self._dct[name] = s
        return s

    def get(self, name):
        s = self._dct.get(name)
        if s is None:
            raise KeyError('no such sprite: %r' % (name,))
        return s

    def all(self):
        return list(self._dct.values())
