_DATA = None


def _thing(proxy, by_name):
    def f(self, x):
        if isinstance(x, proxy):
            return x
        elif isinstance(x, str):
            id = by_name(x)
            return proxy.by_id(id)
        elif isinstance(x, int):
            return proxy.by_id(x)
        else:
            raise TypeError('expected %s, int, or str' % proxy.__name__)
    return f

def _get_thing(proxy, get_by_name):
    def f(self, x):
        if isinstance(x, proxy):
            return x
        elif isinstance(x, str):
            id = get_by_name(x)
            return proxy.by_id(id) if id is not None else None
        elif isinstance(x, int):
            if 0 <= x < len(proxy.INSTANCES):
                return proxy.by_id(x)
            else:
                return None
        elif x is None:
            return None
        else:
            raise TypeError('expected %s, int, or str' % proxy.__name__)
    return f

def _thing_id(proxy, by_name):
    def f(self, x):
        if isinstance(x, proxy):
            return x.id
        elif isinstance(x, str):
            return by_name(x)
        elif isinstance(x, int):
            if x < 0 or x >= len(proxy.INSTANCES):
                raise IndexError(x)
            return x
        else:
            raise TypeError('expected %s, int, or str' % proxy.__name__)
    return f

def _num_things(proxy):
    def f(self):
        return len(proxy.INSTANCES)
    return f

def _define_methods(cls, proxy, thing, things=None):
    things = things or thing + 's'
    by_name = getattr(_DATA, '%s_by_name' % thing)
    get_by_name = getattr(_DATA, 'get_%s_by_name' % thing)

    setattr(cls, '%s' % thing, _thing(proxy, by_name))
    setattr(cls, 'get_%s' % thing, _get_thing(proxy, get_by_name))
    setattr(cls, 'num_%s' % things, _num_things(proxy))
    setattr(cls, '%s_id' % thing, _thing_id(proxy, by_name))

# The DataProxy class and object need to be available "early", that is, at
# import time (since other outpost_server.core modules import its value then).
# But _def_funcs can't be called until _DATA and the DefProxy classes have been
# created.  So we define an empty DataProxy class, and only populate it with
# methods during init().

class DataProxy:
    @classmethod
    def _init(cls):
        _define_methods(cls, BlockProxy, 'block')
        _define_methods(cls, ItemProxy, 'item')
        _define_methods(cls, RecipeProxy, 'recipe')
        _define_methods(cls, TemplateProxy, 'template')
        _define_methods(cls, AnimationProxy, 'animation')
        _define_methods(cls, SpriteLayerProxy, 'sprite_layer')

DATA = DataProxy()


class DefProxy:
    def __init__(self, id):
        self.id = id

    INSTANCES = []

    @classmethod
    def by_id(cls, id):
        if cls.INSTANCES[id] is None:
            cls.INSTANCES[id] = cls(id)
        return cls.INSTANCES[id]

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        try:
            # `self.name` isn't guaranteed to be present, but we'd like to use
            # it if it is.
            return '<%s #%d %r>' % (type(self).__name__, self.id, self.name)
        except Exception:
            return '<%s #%d>' % (type(self).__name__, self.id)

class BlockProxy(DefProxy):
    @property
    def name(self):
        return _DATA.block_name(self.id)

    @property
    def shape(self):
        return _DATA.block_shape(self.id)

class ItemProxy(DefProxy):
    @property
    def name(self):
        return _DATA.item_name(self.id)

class RecipeProxy(DefProxy):
    @property
    def name(self):
        return _DATA.recipe_name(self.id)

    @property
    def station(self):
        id = _DATA.recipe_station(self.id)
        return TemplateProxy.by_id(id)

    @property
    def inputs(self):
        dct = _DATA.recipe_inputs(self.id)
        return {ItemProxy.by_id(k): v for k, v in dct.items()}

    @property
    def outputs(self):
        dct = _DATA.recipe_outputs(self.id)
        return {ItemProxy.by_id(k): v for k, v in dct.items()}

class TemplateProxy(DefProxy):
    @property
    def name(self):
        return _DATA.template_name(self.id)

    @property
    def layer(self):
        return _DATA.template_layer(self.id)

class AnimationProxy(DefProxy):
    @property
    def name(self):
        return _DATA.animation_name(self.id)

    @property
    def framerate(self):
        return _DATA.animation_framerate(self.id)

    @property
    def length(self):
        return _DATA.animation_length(self.id)

class SpriteLayerProxy(DefProxy):
    @property
    def name(self):
        return _DATA.sprite_layer_name(self.id)


def init(data):
    global _DATA
    _DATA = data

    BlockProxy.INSTANCES = [None] * data.block_count()
    ItemProxy.INSTANCES = [None] * data.item_count()
    RecipeProxy.INSTANCES = [None] * data.recipe_count()
    TemplateProxy.INSTANCES = [None] * data.template_count()
    AnimationProxy.INSTANCES = [None] * data.animation_count()
    SpriteLayerProxy.INSTANCES = [None] * data.sprite_layer_count()

    DataProxy._init()
