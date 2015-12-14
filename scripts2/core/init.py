import sys

DATA = None
_DATA = None

class DataProxy(object):
    def __init__(self):
        pass

    def item(self, name):
        id = _DATA.item_by_name(name)
        return ItemProxy.by_id(id)

    def recipe(self, name):
        id = _DATA.recipe_by_name(name)
        return RecipeProxy.by_id(id)

    def template(self, name):
        id = _DATA.template_by_name(name)
        return TemplateProxy.by_id(id)

@classmethod
def _by_id(cls, id):
    if cls.INSTANCES[id] is None:
        cls.INSTANCES[id] = cls(id)
    return cls.INSTANCES[id]


class ItemProxy(object):
    def __init__(self, id):
        self._id = id

    INSTANCES = []
    by_id = _by_id

    def __hash__(self):
        return hash(self._id)

    def __repr__(self):
        return '<item #%d %r>' % (self._id, self.name)

    def id(self):
        return self._id

    @property
    def name(self):
        return _DATA.item_name(self._id)


class RecipeProxy(object):
    def __init__(self, id):
        self._id = id

    INSTANCES = []
    by_id = _by_id

    def __hash__(self):
        return hash(self._id)

    def __repr__(self):
        return '<recipe #%d %r>' % (self._id, self.name)

    def id(self):
        return self._id

    @property
    def name(self):
        return _DATA.recipe_name(self._id)

    @property
    def station(self):
        id = _DATA.recipe_station(self._id)
        return TemplateProxy.by_id(id)

    @property
    def inputs(self):
        dct = _DATA.recipe_inputs(self._id)
        return {ItemProxy.by_id(k): v for k, v in dct.items()}

    @property
    def outputs(self):
        dct = _DATA.recipe_outputs(self._id)
        return {ItemProxy.by_id(k): v for k, v in dct.items()}


class TemplateProxy(object):
    def __init__(self, id):
        self._id = id

    INSTANCES = []
    by_id = _by_id

    def __hash__(self):
        return hash(self._id)

    def __repr__(self):
        return '<template #%d %r>' % (self._id, self.name)

    def id(self):
        return self._id

    @property
    def name(self):
        return _DATA.recipe_name(self._id)


def startup(eng):
    sys.stderr.write('hello %s\n' % eng)
    sys.stderr.write('hello %s\n' % DATA.recipe('anvil'))
    sys.stderr.write('hello %s\n' % eng.now())
    sys.stderr.flush()

def init(storage, data, hooks):
    global _DATA, DATA
    _DATA = data
    DATA = DataProxy()

    ItemProxy.INSTANCES = [None] * data.item_count()
    RecipeProxy.INSTANCES = [None] * data.recipe_count()
    TemplateProxy.INSTANCES = [None] * data.template_count()

    hooks.set_startup(startup)
