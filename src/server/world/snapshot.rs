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
}

pub struct Inner {
    extra: Option<Extra>,

    // Use HashMap instead of VecMap because we expect these maps to be sparse.
    clients: HashMap<ClientId, Client>,
    entities: HashMap<EntityId, Entity>,
    inventories: HashMap<InventoryId, Inventory>,
    planes: HashMap<PlaneId, Plane>,
    terrain_chunks: HashMap<TerrainChunkId, TerrainChunk>,
    structures: HashMap<StructureId, Structure>,
}

macro_rules! snapshot_impls {
    ($(
            object $Obj:ty {
                id $ObjId:ty;
                map $objs:ident;
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
            )*
        }

        impl Inner {
            $(
                pub fn $record_obj(&mut self, id: $ObjId, obj: &$Obj) {
                    self.$objs.insert(id, obj.clone());
                }
            )*
        }
    };
}

snapshot_impls! {
    object Client {
        id ClientId;
        map clients;
        record
            record_client,
            maybe_record_client;
    }

    object Entity {
        id EntityId;
        map entities;
        record
            record_entity,
            maybe_record_entity;
    }

    object Inventory {
        id InventoryId;
        map inventories;
        record
            record_inventory,
            maybe_record_inventory;
    }

    object Plane {
        id PlaneId;
        map planes;
        record
            record_plane,
            maybe_record_plane;
    }

    object TerrainChunk {
        id TerrainChunkId;
        map terrain_chunks;
        record
            record_terrain_chunk,
            maybe_record_terrain_chunk;
    }

    object Structure {
        id StructureId;
        map structures;
        record
            record_structure,
            maybe_record_structure;
    }
}
