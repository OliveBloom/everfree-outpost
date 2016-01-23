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


class Blocks(Objects):
    def create(self, name, shape, tiles):
        b = block.BlockDef(name, shape, tiles)
        self._add(b)
        self.owner.blocks.append(b)
        return self

    def light(self, color, radius):
        self._foreach(lambda s: s.set_light(color, radius))
        return self

class Structures(Objects2):
    def create(self, name, image, model, shape, layer):
        if isinstance(image, structure.StaticAnimDef):
            image = image2.Anim(
                    [image2.Image.from_raw(f) for f in image.frames],
                    image.framerate, image.oneshot)
        else:
            image = image2.Image.from_raw(image)
        self._builder.new(name) \
                .image(image) \
                .mesh(model.to_mesh()) \
                .shape(shape) \
                .layer(layer)
        return self

    def light(self, pos, color, radius):
        self._builder.light(pos, color, radius)
        return self

class Items(Objects2):
    def create(self, name, ui_name, image):
        self._builder.new(name) \
                .display_name(ui_name) \
                .icon(image2.Image.from_raw(image))
        return self

    def recipe(self, station, inputs, count=1):
        for n in self._builder.keys():
            r = builder2.RECIPE.from_item(self._builder[n]) \
                    .station(station)
            for k,v in inputs.items():
                r.input(k, v)
        return self

class Recipes(Objects):
    def create(self, name, ui_name, station, inputs, outputs):
        r = recipe.RecipeDef(name, ui_name, station, inputs, outputs)
        self._add(r)
        self.owner.recipes.append(r)
        return self

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

class Extras(Objects):
    def create(self, name, func):
        e = extra.ExtraDef(name, func)
        self._add(e)
        self.owner.extras.append(e)
        return self


class Builder(object):
    def __init__(self):
        self.blocks = []
        self.recipes = []
        self.anim_groups = []
        self.animations = []
        self.sprites = []
        self.attach_slots = []
        self.loot_tables = []
        self.extras = []


    def block_builder(self):
        return Blocks(self)

    def mk_block(self, *args, **kwargs):
        return self.block_builder().create(*args, **kwargs)


    def structure_builder(self):
        return Structures(builder2.STRUCTURE.child())

    def mk_structure(self, *args, **kwargs):
        return self.structure_builder().create(*args, **kwargs)


    def item_builder(self):
        return Items(builder2.ITEM.child())

    def mk_item(self, *args, **kwargs):
        return self.item_builder().create(*args, **kwargs)


    def recipe_builder(self):
        return Recipes(self)

    def mk_recipe(self, *args, **kwargs):
        return self.recipe_builder().create(*args, **kwargs)


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


    def extra_builder(self):
        return Extras(self)

    def mk_extra(self, *args, **kwargs):
        return self.extra_builder().create(*args, **kwargs)


INSTANCE = Builder()
mk_block = INSTANCE.mk_block
mk_structure = INSTANCE.mk_structure
mk_item = INSTANCE.mk_item
mk_recipe = INSTANCE.mk_recipe
mk_anim_group = INSTANCE.mk_anim_group
mk_sprite = INSTANCE.mk_sprite
mk_attach_slot = INSTANCE.mk_attach_slot
mk_extra = INSTANCE.mk_extra

block_builder = INSTANCE.block_builder
structure_builder = INSTANCE.structure_builder
item_builder = INSTANCE.item_builder
recipe_builder = INSTANCE.recipe_builder
anim_group_builder = INSTANCE.anim_group_builder
sprite_builder = INSTANCE.sprite_builder
attach_slot_builder = INSTANCE.attach_slot_builder
extra_builder = INSTANCE.extra_builder
