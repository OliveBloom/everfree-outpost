from collections import namedtuple

from _outpost_server import V3

ClientId = namedtuple('ClientId', ('raw',))
EntityId = namedtuple('EntityId', ('raw',))
InventoryId = namedtuple('InventoryId', ('raw',))
PlaneId = namedtuple('PlaneId', ('raw',))
TerrainChunkId = namedtuple('TerrainChunkId', ('raw',))
StructureId = namedtuple('StructureId', ('raw',))

StableClientId = namedtuple('StableClientId', ('raw',))
StableEntityId = namedtuple('StableEntityId', ('raw',))
StableInventoryId = namedtuple('StableInventoryId', ('raw',))
StablePlaneId = namedtuple('StablePlaneId', ('raw',))
StableTerrainChunkId = namedtuple('StableTerrainChunkId', ('raw',))
StableStructureId = namedtuple('StableStructureId', ('raw',))
