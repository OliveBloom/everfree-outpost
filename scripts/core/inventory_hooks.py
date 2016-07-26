from outpost_server.core.engine import InventoryProxy


HOOKS = {}

def register(name):
    def decorator(f):
        assert name not in HOOKS, 'duplicate inventory change hook: %r' % name
        HOOKS[name] = f
        return f
    return decorator

def inventory_change_hook(eng, iid):
    i = InventoryProxy(eng, iid)
    spec = i.extra()['special']
    name = spec['name']
    args = spec.get('args', []).copy()
    kwargs = spec.get('kwargs', {}).copy()
    HOOKS[name](i, *args, **kwargs)

def init(hooks):
    hooks.inventory_change_hook(inventory_change_hook)
