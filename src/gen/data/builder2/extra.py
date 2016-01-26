from outpost_data.core.builder2.base import *
from outpost_data.core.consts import *
from outpost_data.core.extra import ExtraDef

class ExtraPrototype(PrototypeBase):
    KIND = 'extra'
    FIELDS = ('func',)

    def instantiate(self):
        name = self.require('name') or '_%x' % id(self)
        func = self.require('func') or (lambda maps: None)

        return ExtraDef(name, func)

class ExtraBuilder(BuilderBase):
    PROTO_CLASS = ExtraPrototype

    func = dict_modifier('func')
