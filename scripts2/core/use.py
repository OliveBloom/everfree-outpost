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
    else:
        print('PASS THROUGH: structure', s.template())
        eng.script_cb_interact(cid, args)

def client_use_item(eng, cid, item_id, args):
    item = ItemProxy.by_id(item_id)
    handler = _USE_ITEM.get(alias.item(item))
    if handler is not None:
        e = ClientProxy(eng, cid).pawn()
        if e.inv('main').count(item) == 0:
            return

        handler(e, args)
    else:
        print('PASS THROUGH: item', item)
        eng.script_cb_use_item(cid, item, args)

def client_use_ability(eng, cid, ability_id, args):
    ability = ItemProxy.by_id(ability_id)
    handler = _USE_ABILITY.get(alias.item(ability))
    if handler is not None:
        e = ClientProxy(eng, cid).pawn()
        if e.inv('ability').count(ability) == 0:
            return

        handler(e, args)
    else:
        print('PASS THROUGH: ability', ability)
        eng.script_cb_use_ability(cid, ability, args)


def structure(name):
    """Decorator for registering structure use handlers.

    Usage:
        @structure('anvil')
        def anvil(client, structure, args): ...
    """
    template = DATA.template(name)
    def register(f):
        _USE_STRUCTURE[template] = f
        return f
    return register

def item(name):
    item = DATA.item(name)
    def register(f):
        _USE_ITEM[item] = f
        return f
    return register

def ability(name):
    ability = DATA.item(name)
    def register(f):
        _USE_ABILITY[ability] = f
        return f
    return register


def init(hooks):
    hooks.client_interact(client_interact)
    hooks.client_use_item(client_use_item)
    hooks.client_use_ability(client_use_ability)
