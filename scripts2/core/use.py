from outpost_server.core import alias, util
from outpost_server.core.data import DATA
from outpost_server.core.data import ItemProxy
from outpost_server.core.engine import ClientProxy

_USE_STRUCTURE = {}
_USE_ITEM = {}
_USE_ABILITY = {}


def client_interact(eng, cid, args):
    e = ClientProxy(eng, cid).pawn()
    s = util.hit_structure(e)
    if s is None:
        return

    handler = _USE_STRUCTURE.get(alias.template(s.template()))
    if handler is not None:
        handler(e, s, args)

def client_use_item(eng, cid, item_id, args):
    item = ItemProxy.by_id(item_id)
    handler = _USE_ITEM.get(alias.item(item))
    if handler is not None:
        e = ClientProxy(eng, cid).pawn()
        if e.inv('main').count(item) == 0:
            return

        handler(e, args)

def client_use_ability(eng, cid, ability_id, args):
    ability = ItemProxy.by_id(ability_id)
    handler = _USE_ABILITY.get(alias.item(ability))
    if handler is not None:
        e = ClientProxy(eng, cid).pawn()
        if e.inv('ability').count(ability) == 0:
            return

        handler(e, args)


# Provide a way to call handlers from outside this module, so that one handler
# can easily dispatch to another.
def call_structure_handler(template, e, s, args):
    handler = _USE_STRUCTURE.get(alias.template(template))
    if handler is not None:
        handler(e, s, args)

def call_item_handler(item, e, args):
    handler = _USE_ITEM.get(alias.item(item))
    if handler is not None:
        handler(e, args)

def call_ability_handler(ability, e, args):
    handler = _USE_ABILITY.get(alias.item(ability))
    if handler is not None:
        handler(e, args)


def structure(name):
    """Decorator for registering structure use handlers.

    Usage:
        @structure('anvil')
        def anvil(client, structure, args): ...
    """
    template = DATA.template(name)
    def register(f):
        assert template not in _USE_STRUCTURE, \
                'duplicate registration for %s (original was %s)' % \
                (template, _USE_STRUCTURE[template].__qualname__)
        _USE_STRUCTURE[template] = f
        return f
    return register

def item(name):
    item = DATA.item(name)
    def register(f):
        assert item not in _USE_ITEM, \
                'duplicate registration for %s (original was %s)' % \
                (item, _USE_ITEM[item].__qualname__)
        _USE_ITEM[item] = f
        return f
    return register

def ability(name):
    ability = DATA.item(name)
    def register(f):
        assert ability not in _USE_ITEM, \
                'duplicate registration for %s (original was %s)' % \
                (ability, _USE_ITEM[ability].__qualname__)
        _USE_ABILITY[ability] = f
        return f
    return register


def init(hooks):
    hooks.client_interact(client_interact)
    hooks.client_use_item(client_use_item)
    hooks.client_use_ability(client_use_ability)
