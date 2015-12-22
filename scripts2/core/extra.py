from outpost_server.core.types import *

class ExtraHashProxy(object):
    def __init__(self, ref):
        assert ref.get_type() == 'hash'
        self._ref = ref

    def __repr__(self):
        if self._ref.is_valid():
            expired = ''
        else:
            expired = ' (expired)'
        return '<ExtraHashProxy at 0x%x for Extra at 0x%x%s>' % (
                id(self), self._ref.get_ptr(), expired)

    def copy(self):
        dct = self._ref.get_hash()
        for k,v in dct:
            dct[k] = ref_get(v)
        return dct

    def __getitem__(self, key):
        return ref_get(self._ref.hash_get(key))

    def __setitem__(self, key, val):
        ref_set(self._ref.hash_put(key), val)

    def __delitem__(self, key):
        self._ref.hash_delete(key)

    def __contains__(self, key):
        return self._ref.hash_contains(key)

    def __len__(self):
        return self._ref.hash_len()

    def __iter__(self):
        return self.keys()

    def get(self, key, default=None):
        if key not in self:
            return default
        else:
            return self[key]

    def setdefault(self, key, default=None):
        if key not in self:
            self[key] = default
        return self[key]

    def keys(self):
        return self._ref.get_hash().keys()

    def values(self):
        return (ref_get(v) for v in self._ref.get_hash().values())

    def items(self):
        return ((k, ref_get(v)) for k,v in self._ref.get_hash().items())

class ExtraArrayProxy(object):
    def __init__(self, ref):
        assert ref.get_type() == 'array'
        self._ref = ref

    def __repr__(self):
        if self._ref.is_valid():
            expired = ''
        else:
            expired = ' (expired)'
        return '<ExtraArrayProxy at 0x%x for Extra at 0x%x%s>' % (
                id(self), self._ref.get_ptr(), expired)

    def copy(self):
        lst = self._ref.get_array()
        for i, v in enumerate(lst):
            lst[i] = ref_get(v)
        return lst

    def __getitem__(self, idx):
        if idx < 0:
            idx = self._ref.array_len() + idx
        return ref_get(self._ref.array_get(idx))

    def __setitem__(self, idx, val):
        if idx < 0:
            idx = self._ref.array_len() + idx
        ref_set(self._ref.array_get(idx), val)

    def __len__(self):
        return self._ref.array_len()

    def __contains__(self, val):
        return self.copy().__contains__(val)

    def __iter__(self):
        return (ref_get(v) for v in self._ref.get_array())

    def append(self, val):
        ref_set(self._ref.array_append(), val)

    def pop(self):
        last = self[-1]
        self._ref.array_pop()
        return last


GET_MAP = {
        'null': lambda ref: None,
        'bool': lambda ref: ref.get_bool(),
        'int': lambda ref: ref.get_int(),
        'float': lambda ref: ref.get_float(),
        'str': lambda ref: ref.get_str(),

        'array': ExtraArrayProxy,
        'hash': ExtraHashProxy,

        'client_id': lambda ref: ClientId(ref.get_client_id()),
        'entity_id': lambda ref: EntityId(ref.get_entity_id()),
        'inventory_id': lambda ref: InventoryId(ref.get_inventory_id()),
        'plane_id': lambda ref: PlaneId(ref.get_plane_id()),
        'terrain_chunk_id': lambda ref: TerrainChunkId(ref.get_terrain_chunk_id()),
        'structure_id': lambda ref: StructureId(ref.get_structure_id()),

        'stable_client_id': lambda ref: StableClientId(ref.get_stable_client_id()),
        'stable_entity_id': lambda ref: StableEntityId(ref.get_stable_entity_id()),
        'stable_inventory_id': lambda ref: StableInventoryId(ref.get_stable_inventory_id()),
        'stable_plane_id': lambda ref: StablePlaneId(ref.get_stable_plane_id()),
        'stable_terrain_chunk_id': lambda ref:
                StableTerrainChunkId(ref.get_stable_terrain_chunk_id()),
        'stable_structure_id': lambda ref: StableStructureId(ref.get_stable_structure_id()),

        'v2': lambda ref: ref.get_v2(),
        'v3': lambda ref: ref.get_v3(),
        'region2': lambda ref: ref.get_region2(),
        'region3': lambda ref: ref.get_region3(),
        }

def ref_get(ref):
    return GET_MAP[ref.get_type()](ref)


def ref_set_array(ref, lst):
    ref.set_array()
    for x in lst:
        subref = ref.list_append()
        ref_set(subref, x)

def ref_set_hash(ref, dct):
    ref.set_hash()
    for k,v in dct.items():
        subref = ref.hash_put(k)
        ref_set(subref, v)

SET_MAP = {id(k): v for k,v in (
        (type(None), lambda r,v: r.set_null()),
        (bool, lambda r,v: r.set_bool(v)),
        (int, lambda r,v: r.set_int(v)),
        (float, lambda r,v: r.set_float(v)),
        (str, lambda r,v: r.set_str(v)),

        (list, ref_set_array),
        (tuple, ref_set_array),
        (dict, ref_set_hash),

        (ClientId, lambda r,v: r.set_client_id(v.raw)),
        (ClientId, lambda r,v: r.set_client_id(v.raw)),
        (EntityId, lambda r,v: r.set_entity_id(v.raw)),
        (InventoryId, lambda r,v: r.set_inventory_id(v.raw)),
        (PlaneId, lambda r,v: r.set_plane_id(v.raw)),
        (TerrainChunkId, lambda r,v: r.set_terrain_chunk_id(v.raw)),
        (StructureId, lambda r,v: r.set_structure_id(v.raw)),

        (StableClientId, lambda r,v: r.set_stable_client_id(v.raw)),
        (StableEntityId, lambda r,v: r.set_stable_entity_id(v.raw)),
        (StableInventoryId, lambda r,v: r.set_stable_inventory_id(v.raw)),
        (StablePlaneId, lambda r,v: r.set_stable_plane_id(v.raw)),
        (StableTerrainChunkId, lambda r,v: r.set_stable_terrain_chunk_id(v.raw)),
        (StableStructureId, lambda r,v: r.set_stable_structure_id(v.raw)),

        # TODO: V2, Region2, Region3
        (V3, lambda r,v: r.set_v3(v)),
        )}

def ref_set(ref, val):
    f = SET_MAP.get(id(type(val)))
    if f is None:
        raise TypeError("don't know how to store %r" % type(val))
    f(ref, val)
