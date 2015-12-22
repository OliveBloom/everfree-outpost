from outpost_server.core.types import *

GET_MAP = {
        'null': lambda ref: None,
        'bool': lambda ref: ref.get_bool(),
        'int': lambda ref: ref.get_int(),
        'float': lambda ref: ref.get_float(),
        'str': lambda ref: ref.get_str(),

        'array': lambda ref: ref.get_array(),
        'hash': lambda ref: ref.get_hash(),

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

GET_ITEM_MAP = dict(GET_MAP, **{
    'array': lambda ref: EntityProxy(ref),
    'hash': lambda ref: EntityProxy(ref),
    })

def ref_get_item(ref):
    return GET_ITEM_MAP[ref.get_type()](ref)


SET_MAP = {id(k): v for k,v in (
        (type(None), lambda r,v: r.set_null()),
        (bool, lambda r,v: r.set_bool(v)),
        (int, lambda r,v: r.set_int(v)),
        (float, lambda r,v: r.set_float(v)),
        (str, lambda r,v: r.set_str(v)),

        # list and dict need special handling

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
    if f is not None:
        f(ref, val)
    else:
        raise TypeError("don't know how to store %r" % type(val))


class ExtraProxy(object):
    def __init__(self, ref):
        self._ref = ref

    def __repr__(self):
        if self._ref.is_valid():
            kind = self._ref.get_type()
        else:
            kind = 'expired'
        return '<ExtraProxy at 0x%x for Extra (%s) at 0x%x>' % (
                id(self), kind, self._ref.get_ptr())

    def get(self):
        return ref_get(self._ref)

    def set(self, val):
        return ref_set(self._ref, val)

    def __getitem__(self, key):
        print(repr(key), type(key))
        if isinstance(key, str):
            subref = self._ref.hash_get(key)
            if subref is None:
                raise KeyError(key)
        else:
            subref = self._ref.array_get(key)
        return ref_get_item(subref)

    def __setitem__(self, key, val):
        if isinstance(key, str):
            subref = self._ref.hash_put(key)
        else:
            subref = self._ref.array_get(key)
        return ref_set(subref, val)

    def __delitem__(self, key):
        if isinstance(key, str):
            subref = self._ref.hash_delete(key)
        else:
            raise TypeError('not a Hash')

    def __len__(self):
        self._ref.array_len()

    def append(self, val):
        subref = self._ref.array_append()
        ref_set(subref, val)

    def pop(self):
        self._ref.array_pop()

    def get_type(self):
        return self._ref.get_type()
