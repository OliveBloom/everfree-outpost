from outpost_server.core.engine import StructureProxy

def multi_lookup(e, k):
    # k is a list or tuple of strs
    for kk in k:
        if kk not in e:
            return None
        e = e[kk]
    return e

def mk_register(key, hooks):
    if isinstance(key, str):
        if '.' in key:
            key = tuple(key.split('.'))
        else:
            key = (key,)

    def decorator(f):
        STRUCTURE_HOOKS.append((key, f))
        return f
    return decorator

def dispatch(obj, hooks):
    for (k, f) in hooks:
        e = multi_lookup(obj.extra(), k)
        if e is not None:
            print('dispatch import hook %s for %s' % (k, obj))
            f(obj, e)



# List of (key, func) pairs.  The func will be called if the key is present in
# the structure extras.  The key is a tuple of strs, to allow for nested
# lookups.
STRUCTURE_HOOKS = []

def structure(key):
    return mk_register(key, STRUCTURE_HOOKS)

def structure_import_hook(eng, sid):
    s = StructureProxy(eng, sid)
    dispatch(s, STRUCTURE_HOOKS)


def init(hooks):
    hooks.structure_import_hook(structure_import_hook)

