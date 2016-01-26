from . import structure, block, item, recipe, animation, attachment, loot_table, extra
from outpost_data.core import builder2, image2


class Objects(object):
    def __init__(self, owner):
        self.owner = owner
        self.x = {}

    def _add(self, obj):
        self.x[obj.name] = obj

    def filter(self, pred):
        result = type(self)(self.owner)
        result.x = dict((k, v) for (k, v) in self.x.items() if pred(v))
        return result

    def merge(self, other):
        assert(type(self) is type(other))
        for k, v in other.x.items():
            self.x[k] = v

    def _foreach(self, f):
        for v in self.x.values():
            f(v)

    def __getitem__(self, k):
        return self.x[k]

    def unwrap(self):
        assert len(self.x) == 1
        return next(iter(self.x.values()))


class Objects2:
    def __init__(self, builder):
        self._builder = builder

    def merge(self, other):
        assert(type(self) is type(other))
        # bad hack
        for k, v in other._builder._dct.items():
            self._builder._dct[k] = v

    def unwrap(self):
        return self._builder.unwrap()

    def __getitem__(self, k):
        return self._builder[k]

class AnimGroups(Objects):
    def create(self, name):
        g = animation.AnimGroupDef(name)
        self._add(g)
        self.owner.anim_groups.append(g)
        return self

    def add_anim(self, name, length, framerate):
        def go(g):
            g.add_anim(name, length, framerate)
        self._foreach(go)
        return self

    def add_anim_mirror(self, name, orig_name):
        def go(g):
            g.add_anim_mirror(name, orig_name)
        self._foreach(go)
        return self

    def finish(self):
        def go(g):
            g.finish()
            for a in g.anims.values():
                self.owner.animations.append(a)
        self._foreach(go)
        return self

class Sprites(Objects):
    def create(self, name, group, size, images):
        r = animation.SpriteDef(name, group, size, images)
        self._add(r)
        self.owner.sprites.append(r)
        return self

class AttachSlots(Objects):
    def create(self, name, anim_group):
        s = attachment.AttachSlotDef(name, anim_group)
        self._add(s)
        self.owner.attach_slots.append(s)
        return self

    def add_variant(self, name, sprite):
        if isinstance(sprite, Objects):
            sprite = sprite.unwrap()
        def go(s):
            s.add_variant(name, sprite)
        self._foreach(go)
        return self


class Builder(object):
    def __init__(self):
        self.anim_groups = []
        self.animations = []
        self.sprites = []
        self.attach_slots = []


    def anim_group_builder(self):
        return AnimGroups(self)

    def mk_anim_group(self, *args, **kwargs):
        return self.anim_group_builder().create(*args, **kwargs)


    def sprite_builder(self):
        return Sprites(self)

    def mk_sprite(self, *args, **kwargs):
        return self.sprite_builder().create(*args, **kwargs)


    def attach_slot_builder(self):
        return AttachSlots(self)

    def mk_attach_slot(self, *args, **kwargs):
        return self.attach_slot_builder().create(*args, **kwargs)


INSTANCE = Builder()
mk_anim_group = INSTANCE.mk_anim_group
mk_sprite = INSTANCE.mk_sprite
mk_attach_slot = INSTANCE.mk_attach_slot

anim_group_builder = INSTANCE.anim_group_builder
sprite_builder = INSTANCE.sprite_builder
attach_slot_builder = INSTANCE.attach_slot_builder
