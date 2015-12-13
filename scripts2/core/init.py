
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


def init(storage, data):
    global _DATA
    _DATA = data

    ItemProxy.INSTANCES = [None] * data.item_count()
    RecipeProxy.INSTANCES = [None] * data.recipe_count()
    TemplateProxy.INSTANCES = [None] * data.template_count()

    d = DataProxy()
    print(d.item('anvil'))
    print(d.recipe('axe'))
    print(d.recipe('axe').inputs)
    print(d.recipe('axe').station)
