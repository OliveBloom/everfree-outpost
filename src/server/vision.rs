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
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;
use vec_map::VecMap;

use libphysics::{CHUNK_SIZE, TILE_SIZE};

use types::*;
use util::{multimap_insert, multimap_remove};
use util::RefcountedMap;
use util::OptionIterExt;
use util::SmallSet;

use pubsub::{self, PubSub};


pub const VIEW_SIZE: V2 = V2 { x: 5, y: 6 };
pub const VIEW_ANCHOR: V2 = V2 { x: 2, y: 2 };

pub fn vision_region(pos: V3) -> Region<V2> {
    let center = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

    let base = center - VIEW_ANCHOR;
    Region::new(base, base + VIEW_SIZE)
}


type ViewerId = ClientId;
type Location = (PlaneId, V2);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum ViewableId {
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

    viewer_pos: HashMap<ViewerId, (PlaneId, Region<V2>)>,
    entity_pos: HashMap<EntityId, (PlaneId, SmallSet<V2>)>,
    structure_pos: HashMap<StructureId, (PlaneId, SmallSet<V2>)>,
    terrain_chunk_pos: HashMap<TerrainChunkId, (PlaneId, V2)>,

    inv_ps: PubSub<InventoryId, (), ViewerId>,
    viewer_invs: HashMap<ViewerId, Vec<InventoryId>>,
}


/// Hooks for handling vision events.
#[allow(unused_variables)]
pub trait Hooks {
    fn on_entity_appear(&mut self, cid: ClientId, eid: EntityId) {}
    fn on_entity_disappear(&mut self, cid: ClientId, eid: EntityId) {}
    fn on_entity_motion_update(&mut self, cid: ClientId, eid: EntityId) {}
    fn on_entity_appearance_update(&mut self, cid: ClientId, eid: EntityId) {}

    fn on_plane_change(&mut self,
                       cid: ClientId,
                       old_pid: PlaneId,
                       new_pid: PlaneId) {}

    fn on_terrain_chunk_appear(&mut self,
                               cid: ClientId,
                               tcid: TerrainChunkId) {}
    fn on_terrain_chunk_disappear(&mut self,
                                  cid: ClientId,
                                  tcid: TerrainChunkId) {}
    fn on_terrain_chunk_update(&mut self,
                               cid: ClientId,
                               tcid: TerrainChunkId) {}

    fn on_structure_appear(&mut self, cid: ClientId, sid: StructureId) {}
    fn on_structure_disappear(&mut self, cid: ClientId, sid: StructureId) {}
    fn on_structure_template_change(&mut self, cid: ClientId, sid: StructureId) {}

    fn on_inventory_appear(&mut self, cid: ClientId, iid: InventoryId) {}
    fn on_inventory_disappear(&mut self, cid: ClientId, iid: InventoryId) {}
    fn on_inventory_update(&mut self,
                           cid: ClientId,
                           iid: InventoryId,
                           slot_idx: u8) {}
}

pub struct NoHooks;
impl Hooks for NoHooks { }


fn on_viewable_appear<H: Hooks>(cid: ClientId,
                                vid: ViewableId,
                                h: &mut H) {
    match vid {
        ViewableId::Entity(eid) => h.on_entity_appear(cid, eid),
        ViewableId::TerrainChunk(eid) => h.on_terrain_chunk_appear(cid, eid),
        ViewableId::Structure(eid) => h.on_structure_appear(cid, eid),
    }
}

fn on_viewable_disappear<H: Hooks>(cid: ClientId,
                                   vid: ViewableId,
                                   h: &mut H) {
    match vid {
        ViewableId::Entity(eid) => h.on_entity_disappear(cid, eid),
        ViewableId::TerrainChunk(eid) => h.on_terrain_chunk_disappear(cid, eid),
        ViewableId::Structure(eid) => h.on_structure_disappear(cid, eid),
    }
}


// Main implementation

impl Vision {
    pub fn new() -> Vision {
        Vision {
            ps: PubSub::new(),

            viewer_pos: HashMap::new(),
            entity_pos: HashMap::new(),
            structure_pos: HashMap::new(),
            terrain_chunk_pos: HashMap::new(),

            inv_ps: PubSub::new(),
            viewer_invs: HashMap::new(),
        }
    }


    pub fn add_client<H>(&mut self,
                         cid: ClientId,
                         plane: PlaneId,
                         area: Region<V2>,
                         h: &mut H)
            where H: Hooks {
        trace!("{:?} created", cid);
        self.viewer_pos.insert(cid, (PLANE_LIMBO, Region::empty()));
        self.viewer_invs.insert(cid, Vec::new());
        self.set_client_area(cid, plane, area, h);
    }

    pub fn remove_client<H>(&mut self,
                            cid: ClientId,
                            h: &mut H)
            where H: Hooks {
        trace!("{:?} destroyed", cid);
        self.set_client_area(cid, PLANE_LIMBO, Region::empty(), h);

        self.viewer_pos.remove(&cid);
        for iid in self.viewer_invs.remove(&cid).unwrap() {
            self.inv_ps.unsubscribe_publisher(cid, iid,
                                              |_,_| h.on_inventory_disappear(cid, iid));
        }
    }

    pub fn set_client_area<H>(&mut self,
                              cid: ClientId,
                              new_plane: PlaneId,
                              new_area: Region<V2>,
                              h: &mut H)
            where H: Hooks {
        let &(old_plane, old_area) = unwrap_or!(self.viewer_pos.get(&cid));
        let plane_change = old_plane != new_plane;

        // Send all "disappear" events first, then all "appear" events.  This prevents the client
        // from seeing a mix of old and new structures in the same place.

        for p in old_area.points().filter(|&p| !new_area.contains(p) || plane_change) {
            if old_plane != PLANE_LIMBO {
                self.ps.unsubscribe(cid, (old_plane, p),
                                    |&vid, _, _| on_viewable_disappear(cid, vid, h));
            }
        }

        if plane_change {
            h.on_plane_change(cid, old_plane, new_plane);
        }

        for p in new_area.points().filter(|&p| !old_area.contains(p) || plane_change) {
            if new_plane != PLANE_LIMBO {
                self.ps.subscribe(cid, (new_plane, p),
                                  |&vid, _, _| on_viewable_appear(cid, vid, h));
            }
        }

        self.viewer_pos.insert(cid, (new_plane, new_area));
    }

    pub fn client_view_plane(&self, cid: ClientId) -> Option<PlaneId> {
        self.viewer_pos.get(&cid).map(|x| x.0)
    }

    pub fn client_view_area(&self, cid: ClientId) -> Option<Region<V2>> {
        self.viewer_pos.get(&cid).map(|x| x.1)
    }


    pub fn add_entity<H>(&mut self,
                         eid: EntityId,
                         plane: PlaneId,
                         area: SmallSet<V2>,
                         h: &mut H)
            where H: Hooks {
        self.entity_pos.insert(eid, (PLANE_LIMBO, SmallSet::new()));
        self.set_entity_area(eid, plane, area, h);
    }

    pub fn remove_entity<H>(&mut self,
                            eid: EntityId,
                            h: &mut H)
            where H: Hooks {
        self.set_entity_area(eid, PLANE_LIMBO, SmallSet::new(), h);
        self.entity_pos.remove(&eid);
    }

    pub fn set_entity_area<H>(&mut self,
                              eid: EntityId,
                              new_plane: PlaneId,
                              new_area: SmallSet<V2>,
                              h: &mut H)
            where H: Hooks {
        let vid = ViewableId::Entity(eid);
        let entry = unwrap_or!(self.entity_pos.get_mut(&eid));
        {
            let old_plane = entry.0;
            let old_area = &entry.1;
            let plane_change = new_plane != old_plane;

            // Send all "appear" events before all "disappear" events.  There are four cases here:
            //  - Neither old nor new position is visible: Refcount remains unchanged (at zero).
            //  - Only old position is visible: First loop has no effect, second decrements
            //    refcount (possibly generating `gone` event).
            //  - Only new position is visible: First loop increments refcount (possibly generating
            //    `appear` event), second has no effect.
            //  - Both old and new are visible: Since old position is visible, refcount is
            //    positive, First loop increments, and second decrements.  No events are generated
            //    because the refcount is positive the whole way through.
            for &p in new_area.iter().filter(|&p| !old_area.contains(p) || plane_change) {
                self.ps.publish(vid, (new_plane, p),
                                |_, _, &cid| h.on_entity_appear(cid, eid));
            }

            for &p in old_area.iter().filter(|&p| !new_area.contains(p) || plane_change) {
                self.ps.unpublish(vid, (old_plane, p),
                                  |_, _, &cid| h.on_entity_disappear(cid, eid));
            }
        }

        *entry = (new_plane, new_area);
        self.ps.message(&vid, |_, &cid| h.on_entity_motion_update(cid, eid));
    }

    pub fn update_entity_appearance<H>(&mut self,
                                       eid: EntityId,
                                       h: &mut H)
            where H: Hooks {
        self.ps.message(&ViewableId::Entity(eid),
                        |_, &cid| h.on_entity_appearance_update(cid, eid));
    }


    pub fn add_terrain_chunk<H>(&mut self,
                                tcid: TerrainChunkId,
                                plane: PlaneId,
                                cpos: V2,
                                h: &mut H)
            where H: Hooks {
        self.terrain_chunk_pos.insert(tcid, (plane, cpos));
        self.ps.publish(ViewableId::TerrainChunk(tcid), (plane, cpos),
                        |_, _, &cid| h.on_terrain_chunk_appear(cid, tcid));
    }

    pub fn remove_terrain_chunk<H>(&mut self,
                                   tcid: TerrainChunkId,
                                   h: &mut H)
            where H: Hooks {
        let loc = unwrap_or!(self.terrain_chunk_pos.remove(&tcid));
        self.ps.unpublish(ViewableId::TerrainChunk(tcid), loc,
                          |_, _, &cid| h.on_terrain_chunk_disappear(cid, tcid));
    }

    pub fn update_terrain_chunk<H>(&mut self,
                                   tcid: TerrainChunkId,
                                   h: &mut H)
            where H: Hooks {
        self.ps.message(&ViewableId::TerrainChunk(tcid),
                        |_, &cid| h.on_terrain_chunk_update(cid, tcid));
    }


    pub fn add_structure<H>(&mut self,
                            sid: StructureId,
                            plane: PlaneId,
                            area: SmallSet<V2>,
                            h: &mut H)
            where H: Hooks {
        self.structure_pos.insert(sid, (PLANE_LIMBO, SmallSet::new()));
        self.set_structure_area(sid, plane, area, h);
    }

    pub fn remove_structure<H>(&mut self,
                               sid: StructureId,
                               h: &mut H)
            where H: Hooks {
        self.set_structure_area(sid, PLANE_LIMBO, SmallSet::new(), h);
        self.structure_pos.remove(&sid);
    }

    pub fn set_structure_area<H>(&mut self,
                                 sid: StructureId,
                                 new_plane: PlaneId,
                                 new_area: SmallSet<V2>,
                                 h: &mut H)
            where H: Hooks {
        let vid = ViewableId::Structure(sid);
        let entry = unwrap_or!(self.structure_pos.get_mut(&sid));
        {
            let old_plane = entry.0;
            let old_area = &entry.1;
            let plane_change = new_plane != old_plane;

            for &p in new_area.iter().filter(|&p| !old_area.contains(p) || plane_change) {
                self.ps.publish(vid, (new_plane, p),
                                |_, _, &cid| h.on_structure_appear(cid, sid));
            }

            for &p in old_area.iter().filter(|&p| !new_area.contains(p) || plane_change) {
                self.ps.unpublish(vid, (old_plane, p),
                                  |_, _, &cid| h.on_structure_disappear(cid, sid));
            }
        }

        *entry = (new_plane, new_area);
    }

    pub fn change_structure_template<H>(&mut self,
                                        sid: StructureId,
                                        h: &mut H)
            where H: Hooks {
        self.ps.message(&ViewableId::Structure(sid),
                        |_, &cid| h.on_structure_template_change(cid, sid));
    }


    pub fn subscribe_inventory<H>(&mut self,
                                  cid: ClientId,
                                  iid: InventoryId,
                                  h: &mut H)
            where H: Hooks {
        let invs = unwrap_or!(self.viewer_invs.get_mut(&cid));
        invs.push(iid);
        self.inv_ps.subscribe_publisher(cid, iid,
                                        |_, _| h.on_inventory_appear(cid, iid));
    }

    pub fn unsubscribe_inventory<H>(&mut self,
                                    cid: ClientId,
                                    iid: InventoryId,
                                    h: &mut H)
            where H: Hooks {
        let invs = unwrap_or!(self.viewer_invs.get_mut(&cid));
        for i in 0 .. invs.len() {
            if invs[i] == iid {
                self.inv_ps.unsubscribe_publisher(cid, iid,
                                                  |_, _| h.on_inventory_disappear(cid, iid));
                invs.swap_remove(i);
                break;
            }
        }
    }

    pub fn update_inventory<H>(&mut self,
                               iid: InventoryId,
                               slot_idx: u8,
                               h: &mut H)
            where H: Hooks {
        self.inv_ps.message(&iid,
                            |_, &cid| h.on_inventory_update(cid, iid, slot_idx));
    }
}


// Fragment

macro_rules! gen_Fragment {
    ($( fn $name:ident($($arg:ident: $arg_ty:ty),*); )*) => {
        pub trait Fragment<'d> {
            type H: Hooks;
            fn with_hooks<F, R>(&mut self, f: F) -> R
                where F: FnOnce(&mut Vision, &mut Self::H) -> R;

            $(
                fn $name(&mut self, $($arg: $arg_ty),*) {
                    self.with_hooks(|sys, hooks| {
                        sys.$name($($arg,)* hooks)
                    })
                }
            )*
        }
    };
}

gen_Fragment! {
    fn add_client(cid: ClientId, plane: PlaneId, view: Region<V2>);
    fn remove_client(cid: ClientId);
    fn set_client_area(cid: ClientId, plane: PlaneId, view: Region<V2>);

    fn add_entity(eid: EntityId, plane: PlaneId, area: SmallSet<V2>);
    fn remove_entity(eid: EntityId);
    fn set_entity_area(eid: EntityId, plane: PlaneId, area: SmallSet<V2>);
    fn update_entity_appearance(eid: EntityId);

    fn add_terrain_chunk(tcid: TerrainChunkId, plane: PlaneId, cpos: V2);
    fn remove_terrain_chunk(tcid: TerrainChunkId);
    fn update_terrain_chunk(tcid: TerrainChunkId);

    fn add_structure(sid: StructureId, plane: PlaneId, area: SmallSet<V2>);
    fn remove_structure(sid: StructureId);
    fn set_structure_area(sid: StructureId, new_plane: PlaneId, new_area: SmallSet<V2>);
    fn change_structure_template(sid: StructureId);

    fn subscribe_inventory(cid: ClientId, iid: InventoryId);
    fn unsubscribe_inventory(cid: ClientId, iid: InventoryId);
    fn update_inventory(iid: InventoryId,
                        slot_idx: u8);
}
