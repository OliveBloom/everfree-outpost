//! Copy-on-write snapshot support for World instances.
//!
//! # Versioning
//!
//! If the World's `snap.version` field is set to X, then for each object, the snapshot contains
//! the most recent state of that object that had `obj.version` <= X.  Note that there may be
//! more than one state with the same version, because the object version is not updated when the
//! object is already known to be newer than the snapshot (because `obj.version > snap.version`).
//!
//! On object creation: the newly created object didn't appear in the snapshot, so its version is
//! `snap.version + 1`.
//!
//! On object mutation: If `obj.version <= snap.version`, set `obj.version` to `snap.version + 1`,
//! and if there is a current snapshot, copy the object state into the snapshot.  Note this
//! involves no extra writes if the object is up to date.  Also note that we update the version
//! even if the snapshot is no longer active, so that the *next* snapshot will recognize that this
//! object changed since the last one.
//!
//! On object deletion: If `obj.version <= snap.version` and there is a current snapshot, copy the
//! object state into the snapshot.  Otherwise (if `obj.version > snap.version`), it means an old
//! version of the object is already in the snapshot, so we don't need to do anything.

use std::collections::HashMap;

use types::*;
use util::BitSlice;

use super::{Client, Entity, Inventory, Plane, TerrainChunk, Structure};
use super::Extra;

pub struct Snapshot {
    version: u32,
    inner: Option<Box<Inner>>,
}

impl Snapshot {
    pub fn new() -> Snapshot {
        Snapshot {
            version: 0,
            inner: None,
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn begin(&mut self) {
        self.version += 1;
        assert!(self.inner.is_none());
        self.inner = Some(Box::new(Inner::new()));
    }

    pub fn active(&self) -> bool {
        self.inner.is_some()
    }

    pub fn end(&mut self) {
        self.inner = None;
    }
}

struct Inner {
    extra: Option<Extra>,

    // Use HashMap instead of VecMap because we expect these maps to be sparse.
    clients: HashMap<ClientId, Client>,
    entities: HashMap<EntityId, Entity>,
    inventories: HashMap<InventoryId, Inventory>,
    planes: HashMap<PlaneId, Plane>,
    terrain_chunks: HashMap<TerrainChunkId, TerrainChunk>,
    structures: HashMap<StructureId, Structure>,

    client_filter: Vec<u8>,
    entity_filter: Vec<u8>,
    inventory_filter: Vec<u8>,
    plane_filter: Vec<u8>,
    terrain_chunk_filter: Vec<u8>,
    structure_filter: Vec<u8>,
}

impl Inner {
    pub fn new() -> Inner {
        Inner {
            extra: None,

            clients: HashMap::new(),
            entities: HashMap::new(),
            inventories: HashMap::new(),
            planes: HashMap::new(),
            terrain_chunks: HashMap::new(),
            structures: HashMap::new(),

            client_filter: Vec::new(),
            entity_filter: Vec::new(),
            inventory_filter: Vec::new(),
            plane_filter: Vec::new(),
            terrain_chunk_filter: Vec::new(),
            structure_filter: Vec::new(),
        }
    }
}

macro_rules! snapshot_impls {
    ($(
            object $Obj:ty {
                id $ObjId:ty;
                map $objs:ident;
                get $get_obj:ident;
                filter $obj_filter:ident,
                    $set_obj_filter:ident,
                    $forget_obj:ident;
                record
                    $record_obj:ident,
                    $maybe_record_obj:ident;
            }
     )*) => {
        impl Snapshot {
            $(
                #[inline]
                pub fn $record_obj(&mut self, id: $ObjId, obj: &$Obj) {
                    if let Some(ref mut inner) = self.inner {
                        inner.$record_obj(id, obj);
                    }
                }

                #[inline]
                pub fn $maybe_record_obj(&mut self, id: $ObjId, obj: &mut $Obj) {
                    if obj.version <= self.version {
                        self.$record_obj(id, obj);
                        obj.version = self.version + 1;
                    }
                }

                pub fn $get_obj(&self, id: $ObjId) -> Option<&$Obj> {
                    if let Some(ref inner) = self.inner {
                        inner.$get_obj(id)
                    } else {
                        None
                    }
                }

                pub fn $set_obj_filter(&mut self, filter: Vec<u8>) {
                    self.inner.as_mut().unwrap().$set_obj_filter(filter);
                }

                pub fn $forget_obj(&mut self, id: $ObjId) {
                    if let Some(ref mut inner) = self.inner {
                        inner.$forget_obj(id);
                    }
                }
            )*
        }

        impl Inner {
            $(
                pub fn $record_obj(&mut self, id: $ObjId, obj: &$Obj) {
                    let should_record = {
                        let idx = id.unwrap() as usize;
                        let bits = BitSlice::from_bytes(&self.$obj_filter);
                        idx < bits.len() && bits.get(idx)
                    };
                    if should_record {
                        self.$record_obj(id, obj);
                    }
                }

                pub fn $get_obj(&self, id: $ObjId) -> Option<&$Obj> {
                    self.$objs.get(&id)
                }

                pub fn $set_obj_filter(&mut self, filter: Vec<u8>) {
                    self.$obj_filter = filter;
                }

                pub fn $forget_obj(&mut self, id: $ObjId) {
                    self.$objs.remove(&id);

                    let idx = id.unwrap() as usize;
                    let bits = BitSlice::from_bytes_mut(&mut self.$obj_filter);
                    if idx < bits.len() && bits.get(idx) {
                        bits.set(idx, false);
                    }
                }
            )*
        }
    };
}

snapshot_impls! {
    object Client {
        id ClientId;
        map clients;
        get get_client;
        filter client_filter,
            set_client_filter,
            forget_client;
        record
            record_client,
            maybe_record_client;
    }

    object Entity {
        id EntityId;
        map entities;
        get get_entity;
        filter entity_filter,
            set_entity_filter,
            forget_entity;
        record
            record_entity,
            maybe_record_entity;
    }

    object Inventory {
        id InventoryId;
        map inventories;
        get get_inventory;
        filter inventory_filter,
            set_inventory_filter,
            forget_inventory;
        record
            record_inventory,
            maybe_record_inventory;
    }

    object Plane {
        id PlaneId;
        map planes;
        get get_plane;
        filter plane_filter,
            set_plane_filter,
            forget_plane;
        record
            record_plane,
            maybe_record_plane;
    }

    object TerrainChunk {
        id TerrainChunkId;
        map terrain_chunks;
        get get_terrain_chunk;
        filter terrain_chunk_filter,
            set_terrain_chunk_filter,
            forget_terrain_chunk;
        record
            record_terrain_chunk,
            maybe_record_terrain_chunk;
    }

    object Structure {
        id StructureId;
        map structures;
        get get_structure;
        filter structure_filter,
            set_structure_filter,
            forget_structure;
        record
            record_structure,
            maybe_record_structure;
    }
}
