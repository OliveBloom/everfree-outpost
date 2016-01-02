from outpost_server.core.types import *
from _outpost_server import ExtraRef, ExtraHashRef, ExtraArrayRef

class ExtraHashProxy(object):
    def __init__(self, ref):
        assert isinstance(ref, (ExtraRef, ExtraHashRef))
        self._ref = ref

    def __repr__(self):
        if self._ref.is_valid():
            expired = ''
        else:
            expired = ' (expired)'
        return '<ExtraHashProxy at 0x%x for %s at 0x%x%s>' % (
                id(self), self._ref.__class__.__name__, id(self._ref), expired)

    def copy(self):
        dct = self._ref.convert()
        for k,v in dct.items():
            dct[k] = wrap_ref(v)
        return dct

    def __getitem__(self, key):
        return wrap_ref(self._ref.get(key))

    def __setitem__(self, key, val):
        put_ref(self._ref, key, val)

    def __delitem__(self, key):
        self._ref.remove(key)

    def __contains__(self, key):
        return self._ref.contains(key)

    def __len__(self):
        return self._ref.len()

    def __iter__(self):
        return self.keys()

    def get(self, key, default=None):
        if key not in self:
            return default
        else:
            return self[key]

    def setdefault(self, key, default=None):
        # NB: Don't return `default` directly.  Otherwise `setdefault(key, {})`
        # would return the original dict rather than the ExtraHashProxy for the
        # newly-added Extra::Hash.
        if key not in self:
            self[key] = default
        return self[key]

    def keys(self):
        return self._ref.convert().keys()

    def values(self):
        return (wrap_ref(v) for v in self._ref.convert().values())

    def items(self):
        return ((k, wrap_ref(v)) for k,v in self._ref.convert().items())


class ExtraArrayProxy(object):
    def __init__(self, ref):
        assert isinstance(ref, ExtraArrayRef)
        self._ref = ref

    def __repr__(self):
        if self._ref.is_valid():
            expired = ''
        else:
            expired = ' (expired)'
        return '<ExtraArrayProxy at 0x%x for %s at 0x%x%s>' % (
                id(self), self._ref.__class__.__name__, id(self._ref), expired)

    def copy(self):
        lst = self._ref.convert()
        for i, x in enumerate(lst):
            lst[i] = wrap_ref(x)
        return lst

    def __getitem__(self, idx):
        if idx < 0:
            idx = self._ref.len() + idx
        return wrap_ref(self._ref.get(idx))

    def __setitem__(self, idx, val):
        if idx < 0:
            idx = self._ref.len() + idx
        put_ref(self._ref, idx, val)

    def __len__(self):
        return self._ref.len()

    def __contains__(self, val):
        return self.copy().__contains__(val)

    def __iter__(self):
        return (wrap_ref(x) for x in self._ref.convert())

    def append(self, val):
        self._ref.push()
        self[-1] = val

    def pop(self):
        last = self[-1]
        self._ref.pop()
        return last


def wrap_ref(x):
    if isinstance(x, ExtraArrayRef):
        return ExtraArrayProxy(x)
    elif isinstance(x, ExtraHashRef):
        return ExtraHashProxy(x)
    else:
        return x

def put_ref(r, k, v):
    if isinstance(v, list) or type(v) is tuple:
        r.set_array(k)
        lst = r.get(k)
        for i, item in enumerate(v):
            lst.push()
            put_ref(lst, i, item)
    elif isinstance(v, dict):
        r.set_hash(k)
        dct = r.get(k)
        for key, val in v.items():
            put_ref(dct, key, val)
    else:
        r.set_value(k, v)
