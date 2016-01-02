from outpost_server.core.data import DATA
from outpost_server.core.engine import ClientProxy

# On import, DATA is not available, so the number of templates/items is not yet
# known.  During `init`, DATA is ready, so these dicts are replaced with lists
# of the correct length.
_USE_STRUCTURE = {}
_USE_ITEM = {}
_USE_ABILITY = {}


def client_interact(eng, cid, args):
    c = ClientProxy(eng, cid)
    e = c.pawn()
    hit_pos = e.pos() + 16 + e.facing() * 32
    s = e.plane().find_structure_at_point(hit_pos.px_to_tile())
    if s is None:
        return

    handler = _USE_STRUCTURE[s.template().id]
    if handler is not None:
        handler(ClientProxy(eng, cid), s, args)
    else:
        eng.script_cb_interact(cid, args)

def client_use_item(eng, cid, item, args):
    handler = _USE_ITEM[item]
    if handler is not None:
        handler(ClientProxy(eng, cid), args)
    else:
        eng.script_cb_use_item(cid, item, args)

def client_use_ability(eng, cid, ability, args):
    handler = _USE_ABILITY[ability]
    if handler is not None:
        handler(ClientProxy(eng, cid), args)
    else:
        eng.script_cb_use_ability(cid, ability, args)

def structure(name):
    """Decorator for registering structure use handlers.

    Usage:
        @structure('anvil')
        def anvil(client, structure, args): ...
    """
    id = DATA.item(name).id
    def register(f):
        _USE_ITEM[id] = f
        return f
    return register

def item(name):
    id = DATA.item(name).id
    def register(f):
        _USE_ITEM[id] = f
        return f
    return register

def ability(name):
    id = DATA.item(name).id
    def register(f):
        _USE_ABILITY[id] = f
        return f
    return register


def init(hooks):
    global _USE_STRUCTURE, _USE_ITEM, _USE_ABILITY
    _USE_STRUCTURE = [_USE_STRUCTURE.get(i) for i in range(DATA.num_templates())]
    _USE_ITEM = [_USE_ITEM.get(i) for i in range(DATA.num_items())]
    _USE_ABILITY = [_USE_ABILITY.get(i) for i in range(DATA.num_items())]

    hooks.client_interact(client_interact)
    hooks.client_use_item(client_use_item)
    hooks.client_use_ability(client_use_ability)
