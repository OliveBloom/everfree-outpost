use std::collections::HashMap;

use types::*;


pub enum Extra {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),

    Array(Vec<Extra>),
    Hash(HashMap<String, Extra>),

    ClientId(ClientId),
    EntityId(EntityId),
    InventoryId(InventoryId),
    PlaneId(PlaneId),
    TerrainChunkId(TerrainChunkId),
    StructureId(StructureId),

    StableClientId(Stable<ClientId>),
    StableEntityId(Stable<EntityId>),
    StableInventoryId(Stable<InventoryId>),
    StablePlaneId(Stable<PlaneId>),
    StableTerrainChunkId(Stable<TerrainChunkId>),
    StableStructureId(Stable<StructureId>),

    V2(V2),
    V3(V3),
    Region2(Region<V2>),
    Region3(Region<V3>),
}

impl Extra {
    pub fn new_hash() -> Extra {
        Extra::Hash(HashMap::new())
    }
}
