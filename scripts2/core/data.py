_DATA = None

class DataProxy(object):
    def __init__(self):
        pass

    # Get the Proxy for a data item.  These functions accept a Proxy, an
    # integer ID, or a string name.
    def item(self, x):
        if isinstance(x, ItemProxy):
            return x
        elif isinstance(x, int):
            return ItemProxy.by_id(x)
        elif isinstance(x, str):
            return self.item_by_name(x)
        raise TypeError('expected ItemProxy, int, or str')

    def recipe(self, x):
        if isinstance(x, RecipeProxy):
            return x
        elif isinstance(x, int):
            return RecipeProxy.by_id(x)
        elif isinstance(x, str):
            return self.recipe_by_name(x)
        raise TypeError('expected RecipeProxy, int, or str')

    def template(self, x):
        if isinstance(x, TemplateProxy):
            return x
        elif isinstance(x, int):
            return TemplateProxy.by_id(x)
        elif isinstance(x, str):
            return self.template_by_name(x)
        raise TypeError('expected TemplateProxy, int, or str')

    # Get the Proxy for a data item if it exists.  These are similar to to the
    # methods above, but return None instead of raising an exception.
    def get_item(self, x):
        if isinstance(x, ItemProxy):
            return x
        elif isinstance(x, int):
            if x >= 0 and x < self.num_items():
                return ItemProxy.by_id(x)
            else:
                return None
        elif isinstance(x, str):
            return ItemProxy.by_id(_DATA.get_item_by_name(x))
        elif x is None:
            return None
        raise TypeError('expected ItemProxy, int, str, or None')

    def get_recipe(self, x):
        if isinstance(x, RecipeProxy):
            return x
        elif isinstance(x, int):
            if x >= 0 and x < self.num_recipes():
                return RecipeProxy.by_id(x)
            else:
                return None
        elif isinstance(x, str):
            return RecipeProxy.by_id(_DATA.get_recipe_by_name(x))
        elif x is None:
            return None
        raise TypeError('expected RecipeProxy, int, str, or None')

    def get_template(self, x):
        if isinstance(x, TemplateProxy):
            return x
        elif isinstance(x, int):
            if x >= 0 and x < self.num_templates():
                return TemplateProxy.by_id(x)
            else:
                return None
        elif isinstance(x, str):
            return TemplateProxy.by_id(_DATA.get_template_by_name(x))
        elif x is None:
            return None
        raise TypeError('expected TemplateProxy, int, str, or None')

    def item_by_name(self, name):
        id = _DATA.item_by_name(name)
        return ItemProxy.by_id(id)

    def recipe_by_name(self, name):
        id = _DATA.recipe_by_name(name)
        return RecipeProxy.by_id(id)

    def template_by_name(self, name):
        id = _DATA.template_by_name(name)
        return TemplateProxy.by_id(id)

    def num_items(self):
        return len(ItemProxy.INSTANCES)

    def num_recipes(self):
        return len(RecipeProxy.INSTANCES)

    def num_templates(self):
        return len(TemplateProxy.INSTANCES)

DATA = DataProxy()


@classmethod
def _by_id(cls, id):
    if cls.INSTANCES[id] is None:
        cls.INSTANCES[id] = cls(id)
    return cls.INSTANCES[id]

class ItemProxy(object):
    def __init__(self, id):
        self.id = id

    INSTANCES = []
    by_id = _by_id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<item #%d %r>' % (self.id, self.name)

    @property
    def name(self):
        return _DATA.item_name(self.id)

class RecipeProxy(object):
    def __init__(self, id):
        self.id = id

    INSTANCES = []
    by_id = _by_id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<recipe #%d %r>' % (self.id, self.name)

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

class TemplateProxy(object):
    def __init__(self, id):
        self.id = id

    INSTANCES = []
    by_id = _by_id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<template #%d %r>' % (self.id, self.name)

    @property
    def name(self):
        return _DATA.template_name(self.id)


def init(data):
    global _DATA
    _DATA = data

    ItemProxy.INSTANCES = [None] * data.item_count()
    RecipeProxy.INSTANCES = [None] * data.recipe_count()
    TemplateProxy.INSTANCES = [None] * data.template_count()
