from outpost_server.core.data import DATA

_TEMPLATE_ALIAS = {}
_ITEM_ALIAS = {}

def register_template(name, target):
    """Register `name` as an alias for `target`.  The logic in `use` will treat
    structures with template `name` as if they instead had template `target`.
    """
    _TEMPLATE_ALIAS[DATA.template(name)] = DATA.template(target)

def register_item(name, target):
    _ITEM_ALIAS[DATA.item(name)] = DATA.item(target)

def template(template):
    """Similar to `DATA.template`, but also resolves aliases."""
    template = DATA.template(template)
    return _TEMPLATE_ALIAS.get(template, template)

def item(item):
    item = DATA.item(item)
    return _ITEM_ALIAS.get(item, item)
