//! Visibility tracking for game objects.  The vision system keeps track of which objects in the
//! world are visible to which clients.  External callers notify the vision system about changes to
//! the state of the world (creation/destruction/movement of game objects or client viewports), and
//! the vision system invokes hooks to notify about changes in visibility (object enters/leaves a
//! particular viewport).
//!
//! The vision system operates only in terms of object IDs.  It relies on external code to provide
//! the relevant information about each object (such as position), rather than trying to extract
//! information directly from the `World`.  Hooks must also consult the `World` for more
//! information about updated game objects, as the vision system itself stores only the bare
//! minimum to track visibility.
//!
//! In the overall server architecture, the vision system acts as a sort of filter between world
//! updates and client messages, ensuring that each client receives updates only for objects it can
//! actually see.
use std::collections::HashMap;

use libphysics::{CHUNK_SIZE, TILE_SIZE};

use types::*;

use pubsub::{self, PubSub};


pub const VIEW_SIZE: V2 = V2 { x: 5, y: 6 };
pub const VIEW_ANCHOR: V2 = V2 { x: 2, y: 2 };

pub fn vision_region(pos: V3) -> Region<V2> {
    let center = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
    vision_region_chunk(center)
}

pub fn vision_region_chunk(cpos: V2) -> Region<V2> {
    let base = cpos - VIEW_ANCHOR;
    Region::new(base, base + VIEW_SIZE)
}


type ViewerId = ClientId;
type Location = (PlaneId, V2);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ViewableId {
    Entity(EntityId),
    TerrainChunk(TerrainChunkId),
    Structure(StructureId),
}

impl pubsub::Name for ViewableId {
    fn min_bound() -> ViewableId { ViewableId::Entity(pubsub::Name::min_bound()) }
    fn max_bound() -> ViewableId { ViewableId::Structure(pubsub::Name::max_bound()) }
}


/// Vision subsystem state
pub struct Vision {
    ps: PubSub<ViewableId, Location, ViewerId>,

    inv_ps: PubSub<InventoryId, (), ViewerId>,
    viewer_invs: HashMap<ViewerId, Vec<InventoryId>>,
}


// Main implementation

impl Vision {
    pub fn new() -> Vision {
        Vision {
            ps: PubSub::new(),

            inv_ps: PubSub::new(),
            viewer_invs: HashMap::new(),
        }
    }


    // In general, client code should perform all "add" operations before all "remove"
    // operations.  There are four cases here:
    //  - Neither old nor new position is visible: Refcount remains unchanged (at zero).
    //  - Only old position is visible: First loop has no effect, second decrements refcount
    //    (possibly generating `gone` event).
    //  - Only new position is visible: First loop increments refcount (possibly generating
    //    `appear` event), second has no effect.
    //  - Both old and new are visible: Since old position is visible, refcount is positive, First
    //    loop increments, and second decrements.  No events are generated because the refcount is
    //    positive the whole way through.

    pub fn client_add<F>(&mut self,
                         cid: ClientId,
                         plane: PlaneId,
                         cpos: V2,
                         mut f: F)
            where F: FnMut(ViewableId) {
        self.ps.subscribe(cid, (plane, cpos),
                          |&id, _ ,_| f(id));
    }

    pub fn client_remove<F>(&mut self,
                            cid: ClientId,
                            plane: PlaneId,
                            cpos: V2,
                            mut f: F)
            where F: FnMut(ViewableId) {
        self.ps.unsubscribe(cid, (plane, cpos),
                            |&id, _ ,_| f(id));
    }

    pub fn entity_add<F>(&mut self,
                         eid: EntityId,
                         plane: PlaneId,
                         cpos: V2,
                         mut f: F)
            where F: FnMut(ClientId) {
        self.ps.publish(ViewableId::Entity(eid), (plane, cpos),
                        |_, _, &cid| f(cid));
    }

    pub fn entity_remove<F>(&mut self,
                            eid: EntityId,
                            plane: PlaneId,
                            cpos: V2,
                            mut f: F)
            where F: FnMut(ClientId) {
        self.ps.unpublish(ViewableId::Entity(eid), (plane, cpos),
                          |_, _, &cid| f(cid));
    }

    pub fn entity_update<F>(&mut self,
                            eid: EntityId,
                            mut f: F)
            where F: FnMut(ClientId) {
        self.ps.message(&ViewableId::Entity(eid),
                        |_, &cid| f(cid));
    }


    pub fn terrain_chunk_add<F>(&mut self,
                                tcid: TerrainChunkId,
                                plane: PlaneId,
                                cpos: V2,
                                mut f: F)
            where F: FnMut(ClientId) {
        self.ps.publish(ViewableId::TerrainChunk(tcid), (plane, cpos),
                        |_, _, &cid| f(cid));
    }

    pub fn terrain_chunk_remove<F>(&mut self,
                                   tcid: TerrainChunkId,
                                   plane: PlaneId,
                                   cpos: V2,
                                   mut f: F)
            where F: FnMut(ClientId) {
        self.ps.unpublish(ViewableId::TerrainChunk(tcid), (plane, cpos),
                          |_, _, &cid| f(cid));
    }

    pub fn terrain_chunk_update<F>(&mut self,
                                   tcid: TerrainChunkId,
                                   mut f: F)
            where F: FnMut(ClientId) {
        self.ps.message(&ViewableId::TerrainChunk(tcid),
                        |_, &cid| f(cid));
    }


    pub fn structure_add<F>(&mut self,
                            sid: StructureId,
                            plane: PlaneId,
                            cpos: V2,
                            mut f: F)
            where F: FnMut(ClientId) {
        self.ps.publish(ViewableId::Structure(sid), (plane, cpos),
                        |_, _, &cid| f(cid));
    }

    pub fn structure_remove<F>(&mut self,
                               sid: StructureId,
                               plane: PlaneId,
                               cpos: V2,
                               mut f: F)
            where F: FnMut(ClientId) {
        self.ps.unpublish(ViewableId::Structure(sid), (plane, cpos),
                          |_, _, &cid| f(cid));
    }

    pub fn structure_update<F>(&mut self,
                               sid: StructureId,
                               mut f: F)
            where F: FnMut(ClientId) {
        self.ps.message(&ViewableId::Structure(sid),
                        |_, &cid| f(cid));
    }


    pub fn subscribe_inventory<F>(&mut self,
                                  cid: ClientId,
                                  iid: InventoryId,
                                  mut f: F)
            where F: FnMut() {
        let invs = unwrap_or!(self.viewer_invs.get_mut(&cid));
        invs.push(iid);
        self.inv_ps.subscribe_publisher(cid, iid,
                                        |_, _| f());
    }

    pub fn unsubscribe_inventory<F>(&mut self,
                                    cid: ClientId,
                                    iid: InventoryId,
                                    mut f: F)
            where F: FnMut() {
        let invs = unwrap_or!(self.viewer_invs.get_mut(&cid));
        for i in 0 .. invs.len() {
            if invs[i] == iid {
                self.inv_ps.unsubscribe_publisher(cid, iid,
                                                  |_, _| f());
                invs.swap_remove(i);
                break;
            }
        }
    }

    pub fn update_inventory<F>(&mut self,
                               iid: InventoryId,
                               mut f: F)
            where F: FnMut(ClientId) {
        self.inv_ps.message(&iid,
                            |_, &cid| f(cid));
    }

    pub fn init_inventory_subscriptions(&mut self, cid: ClientId) {
        self.viewer_invs.insert(cid, Vec::new());
    }

    pub fn purge_inventory_subscriptions(&mut self, cid: ClientId) {
        for iid in self.viewer_invs.remove(&cid).unwrap() {
            self.inv_ps.unsubscribe_publisher(cid, iid,
                                              |_,_| {});
        }
    }
}
