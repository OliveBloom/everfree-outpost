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
use std::collections::{HashMap, HashSet, VecMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;

use libphysics::{CHUNK_SIZE, TILE_SIZE};

use types::*;
use util::{multimap_insert, multimap_remove};
use util::RefcountedMap;
use util::OptionIterExt;
use util::SmallSet;


pub const VIEW_SIZE: V2 = V2 { x: 5, y: 6 };
pub const VIEW_ANCHOR: V2 = V2 { x: 2, y: 2 };

pub fn vision_region(pos: V3) -> Region<V2> {
    let center = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

    let base = center - VIEW_ANCHOR;
    Region::new(base, base + VIEW_SIZE)
}


type ViewerId = ClientId;
type Location = (PlaneId, V2);


/// Generic method of working with viewable objects.
trait Tag {
    type Id: Copy + Eq + Hash + Debug;

    fn raw_id(id: Self::Id) -> u32;
    fn make_id(raw: u32) -> Self::Id;

    fn viewable_map(v: &ViewableMaps) -> &ViewableMap;
    fn viewable_map_mut(v: &mut ViewableMaps) -> &mut ViewableMap;

    fn visible(v: &Viewer) -> &RefcountedMap<Self::Id, ()>;
    fn visible_mut(v: &mut Viewer) -> &mut RefcountedMap<Self::Id, ()>;

    fn on_appear<H: Hooks>(cid: ClientId, id: Self::Id, h: &mut H);
    fn on_disappear<H: Hooks>(cid: ClientId, id: Self::Id, h: &mut H);
}

struct EntityTag;
struct TerrainChunkTag;
struct StructureTag;
// If you add to this, be sure to update code below that maps over every tag.

impl Tag for EntityTag {
    type Id = EntityId;

    fn raw_id(id: EntityId) -> u32 { id.unwrap() }
    fn make_id(raw: u32) -> EntityId { EntityId(raw) }

    fn viewable_map(v: &ViewableMaps) -> &ViewableMap {
        &v.entities
    }
    fn viewable_map_mut(v: &mut ViewableMaps) -> &mut ViewableMap {
        &mut v.entities
    }

    fn visible(v: &Viewer) -> &RefcountedMap<EntityId, ()> {
        &v.visible_entities
    }
    fn visible_mut(v: &mut Viewer) -> &mut RefcountedMap<EntityId, ()> {
        &mut v.visible_entities
    }

    fn on_appear<H: Hooks>(cid: ClientId, eid: EntityId, h: &mut H) {
        h.on_entity_appear(cid, eid);
    }

    fn on_disappear<H: Hooks>(cid: ClientId, eid: EntityId, h: &mut H) {
        h.on_entity_disappear(cid, eid);
    }
}

impl Tag for TerrainChunkTag {
    type Id = TerrainChunkId;

    fn raw_id(id: TerrainChunkId) -> u32 { id.unwrap() }
    fn make_id(raw: u32) -> TerrainChunkId { TerrainChunkId(raw) }

    fn viewable_map(v: &ViewableMaps) -> &ViewableMap {
        &v.terrain_chunks
    }
    fn viewable_map_mut(v: &mut ViewableMaps) -> &mut ViewableMap {
        &mut v.terrain_chunks
    }

    fn visible(v: &Viewer) -> &RefcountedMap<TerrainChunkId, ()> {
        &v.visible_terrain_chunks
    }
    fn visible_mut(v: &mut Viewer) -> &mut RefcountedMap<TerrainChunkId, ()> {
        &mut v.visible_terrain_chunks
    }

    fn on_appear<H: Hooks>(cid: ClientId, eid: TerrainChunkId, h: &mut H) {
        h.on_terrain_chunk_appear(cid, eid);
    }

    fn on_disappear<H: Hooks>(cid: ClientId, eid: TerrainChunkId, h: &mut H) {
        h.on_terrain_chunk_disappear(cid, eid);
    }
}

impl Tag for StructureTag {
    type Id = StructureId;

    fn raw_id(id: StructureId) -> u32 { id.unwrap() }
    fn make_id(raw: u32) -> StructureId { StructureId(raw) }

    fn viewable_map(v: &ViewableMaps) -> &ViewableMap {
        &v.structures
    }
    fn viewable_map_mut(v: &mut ViewableMaps) -> &mut ViewableMap {
        &mut v.structures
    }

    fn visible(v: &Viewer) -> &RefcountedMap<StructureId, ()> {
        &v.visible_structures
    }
    fn visible_mut(v: &mut Viewer) -> &mut RefcountedMap<StructureId, ()> {
        &mut v.visible_structures
    }

    fn on_appear<H: Hooks>(cid: ClientId, eid: StructureId, h: &mut H) {
        h.on_structure_appear(cid, eid);
    }

    fn on_disappear<H: Hooks>(cid: ClientId, eid: StructureId, h: &mut H) {
        h.on_structure_disappear(cid, eid);
    }
}


/// Vision subsystem state
pub struct Vision {
    viewers: VecMap<Viewer>,
    viewers_by_pos: HashMap<Location, HashSet<ViewerId>>,

    maps: ViewableMaps,

    // Inventories have no position, so they don't need a whole ViewableMap.
    inventory_viewers: HashMap<InventoryId, HashSet<ViewerId>>,
}

struct Viewer {
    id: ViewerId,
    plane: PlaneId,
    area: Region<V2>,

    visible_entities: RefcountedMap<EntityId, ()>,
    visible_terrain_chunks: RefcountedMap<TerrainChunkId, ()>,
    visible_structures: RefcountedMap<StructureId, ()>,
    visible_inventories: RefcountedMap<InventoryId, ()>,
}

impl Viewer {
    fn new(id: ViewerId) -> Viewer {
        Viewer {
            id: id,
            plane: PLANE_LIMBO,
            area: Region::empty(),

            visible_entities: RefcountedMap::new(),
            visible_terrain_chunks: RefcountedMap::new(),
            visible_structures: RefcountedMap::new(),
            visible_inventories: RefcountedMap::new(),
        }
    }
}

struct ViewableMaps {
    entities: ViewableMap,
    terrain_chunks: ViewableMap,
    structures: ViewableMap,
}

struct ViewableMap {
    objs: VecMap<Viewable>,
    objs_by_pos: HashMap<Location, HashSet<u32>>,
}

impl ViewableMap {
    fn new() -> ViewableMap {
        ViewableMap {
            objs: VecMap::new(),
            objs_by_pos: HashMap::new(),
        }
    }
}

struct Viewable {
    plane: PlaneId,
    area: SmallSet<V2>,
    viewers: HashSet<ViewerId>,
}

impl Viewable {
    fn new() -> Viewable {
        Viewable {
            plane: PLANE_LIMBO,
            area: SmallSet::new(),
            viewers: HashSet::new(),
        }
    }
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


// Main implementation

/// Mark a single object as visible to a viewer.
fn retain_obj<T: Tag, H: Hooks>(v: &mut Viewer,
                                obj: &mut Viewable,
                                id: T::Id,
                                h: &mut H) {
    let vid = v.id;
    T::visible_mut(v).retain(id, || {
        trace!("{:?}: retained +{:?}", vid, id);
        T::on_appear(vid, id, h);
        obj.viewers.insert(vid);
    });
}

/// Unmark a single object.
fn release_obj<T: Tag, H: Hooks>(v: &mut Viewer,
                                 obj: &mut Viewable,
                                 id: T::Id,
                                 h: &mut H) {
    let vid = v.id;
    T::visible_mut(v).release(id, |()| {
        trace!("{:?}: released -{:?}", vid, id);
        T::on_disappear(vid, id, h);
        obj.viewers.remove(&vid);
    });
}

/// Mark all objects of a certain type in a chunk for a particular viewer.
fn viewer_retain_typed<T: Tag, H: Hooks>(v: &mut Viewer,
                                         vms: &mut ViewableMaps,
                                         loc: Location,
                                         h: &mut H) {
    let vm = T::viewable_map_mut(vms);
    for &id in vm.objs_by_pos.get(&loc).map(|x| x.iter()).unwrap_iter() {
        retain_obj::<T, H>(v, &mut vm.objs[id as usize], T::make_id(id), h);
    }
}

/// Unmark all objects of a certain type in a chunk for a particular viewer.
fn viewer_release_typed<T: Tag, H: Hooks>(v: &mut Viewer,
                                          vms: &mut ViewableMaps,
                                          loc: Location,
                                          h: &mut H) {
    let vm = T::viewable_map_mut(vms);
    for &id in vm.objs_by_pos.get(&loc).map(|x| x.iter()).unwrap_iter() {
        release_obj::<T, H>(v, &mut vm.objs[id as usize], T::make_id(id), h);
    }
}

/// Mark all objects in a chunk for a particular viewer.
fn viewer_retain<H: Hooks>(v: &mut Viewer,
                           vms: &mut ViewableMaps,
                           loc: Location,
                           h: &mut H) {
    viewer_retain_typed::<EntityTag, H>(v, vms, loc, h);
    viewer_retain_typed::<TerrainChunkTag, H>(v, vms, loc, h);
    viewer_retain_typed::<StructureTag, H>(v, vms, loc, h);
}

/// Unmark all objects in a chunk for a particular viewer.
fn viewer_release<H: Hooks>(v: &mut Viewer,
                            vms: &mut ViewableMaps,
                            loc: Location,
                            h: &mut H) {
    viewer_release_typed::<EntityTag, H>(v, vms, loc, h);
    viewer_release_typed::<TerrainChunkTag, H>(v, vms, loc, h);
    viewer_release_typed::<StructureTag, H>(v, vms, loc, h);
}


impl Vision {
    pub fn new() -> Vision {
        Vision {
            viewers: VecMap::new(),
            viewers_by_pos: HashMap::new(),

            maps: ViewableMaps {
                entities: ViewableMap::new(),
                terrain_chunks: ViewableMap::new(),
                structures: ViewableMap::new(),
            },

            inventory_viewers: HashMap::new(),
        }
    }


    pub fn add_client<H>(&mut self,
                         cid: ClientId,
                         plane: PlaneId,
                         area: Region<V2>,
                         h: &mut H)
            where H: Hooks {
        trace!("{:?} created", cid);
        self.viewers.insert(cid.unwrap() as usize, Viewer::new(cid));
        self.set_client_area(cid, plane, area, h);
    }

    pub fn remove_client<H>(&mut self,
                            cid: ClientId,
                            h: &mut H)
            where H: Hooks {
        trace!("{:?} destroyed", cid);
        self.set_client_area(cid, PLANE_LIMBO, Region::empty(), h);

        let raw_cid = cid.unwrap() as usize;
        for (&iid, _) in self.viewers[&raw_cid].visible_inventories.iter() {
            multimap_remove(&mut self.inventory_viewers, iid, cid);
            h.on_inventory_disappear(cid, iid);
        }

        self.viewers.remove(&raw_cid);
    }

    pub fn set_client_area<H>(&mut self,
                              cid: ClientId,
                              new_plane: PlaneId,
                              new_area: Region<V2>,
                              h: &mut H)
            where H: Hooks {
        let raw_cid = cid.unwrap() as usize;
        let viewer = unwrap_or!(self.viewers.get_mut(&raw_cid));
        let old_plane = mem::replace(&mut viewer.plane, new_plane);
        let old_area = mem::replace(&mut viewer.area, new_area);
        let plane_change = old_plane != new_plane;
        let vms = &mut self.maps;

        // Send all "disappear" events first, then all "appear" events.  This prevents the client
        // from seeing a mix of old and new structures in the same place.

        for p in old_area.points().filter(|&p| !new_area.contains(p) || plane_change) {
            let loc = (old_plane, p);
            viewer_release(viewer, vms, loc, h);
            if old_plane != PLANE_LIMBO {
                multimap_remove(&mut self.viewers_by_pos, loc, cid);
            }
        }

        if plane_change {
            h.on_plane_change(cid, old_plane, new_plane);
        }

        for p in new_area.points().filter(|&p| !old_area.contains(p) || plane_change) {
            let loc = (new_plane, p);
            viewer_retain(viewer, vms, loc, h);
            if new_plane != PLANE_LIMBO {
                multimap_insert(&mut self.viewers_by_pos, loc, cid);
            }
        }
    }

    pub fn client_view_plane(&self, cid: ClientId) -> Option<PlaneId> {
        self.viewers.get(&(cid.unwrap() as usize)).map(|c| c.plane)
    }

    pub fn client_view_area(&self, cid: ClientId) -> Option<Region<V2>> {
        self.viewers.get(&(cid.unwrap() as usize)).map(|c| c.area)
    }


    fn add_viewable<T, H>(&mut self,
                          id: T::Id,
                          plane: PlaneId,
                          area: SmallSet<V2>,
                          h: &mut H)
            where T: Tag, H: Hooks {
        trace!("add {:?}", id);
        T::viewable_map_mut(&mut self.maps).objs.insert(T::raw_id(id) as usize,
                                                        Viewable::new());
        self.set_viewable_area::<T, H>(id, plane, area, h);
    }

    fn remove_viewable<T, H>(&mut self,
                             id: T::Id,
                             h: &mut H)
            where T: Tag, H: Hooks {
        trace!("remove {:?}", id);
        self.set_viewable_area::<T, H>(id, PLANE_LIMBO, SmallSet::new(), h);
        T::viewable_map_mut(&mut self.maps).objs.remove(&(T::raw_id(id) as usize));
    }

    fn set_viewable_area<T, H>(&mut self,
                               id: T::Id,
                               new_plane: PlaneId,
                               new_area: SmallSet<V2>,
                               h: &mut H)
            where T: Tag, H: Hooks {
        let vm = T::viewable_map_mut(&mut self.maps);
        let obj = &mut vm.objs[T::raw_id(id) as usize];

        let old_plane = mem::replace(&mut obj.plane, new_plane);
        // SmallSet is non-Copy, so insert a dummy value here and set the real one later.
        let old_area = mem::replace(&mut obj.area, SmallSet::new());
        let plane_change = new_plane != old_plane;

        // Send all "appear" events before all "disappear" events.  There are four cases here:
        //  - Neither old nor new position is visible: Refcount remains unchanged (at zero).
        //  - Only old position is visible: First loop has no effect, second decrements refcount
        //    (possibly generating `gone` event).
        //  - Only new position is visible: First loop increments refcount (possibly generating
        //    `appeear` event), second has no effect.
        //  - Both old and new are visible: Since old position is visible, refcount is positive,
        //    First loop increments, and second decrements.  No events are generated because the
        //    refcount is positive the whole way through.
        for &p in new_area.iter().filter(|&p| !old_area.contains(p) || plane_change) {
            let loc = (new_plane, p);
            for &vid in self.viewers_by_pos.get(&loc).map(|x| x.iter()).unwrap_iter() {
                let v = &mut self.viewers[vid.unwrap() as usize];
                retain_obj::<T, H>(v, obj, id, h);
            }
            if new_plane != PLANE_LIMBO {
                multimap_insert(&mut vm.objs_by_pos, loc, T::raw_id(id));
            }
        }

        for &p in old_area.iter().filter(|&p| !new_area.contains(p) || plane_change) {
            let loc = (old_plane, p);
            for &vid in self.viewers_by_pos.get(&loc).map(|x| x.iter()).unwrap_iter() {
                let v = &mut self.viewers[vid.unwrap() as usize];
                release_obj::<T, H>(v, obj, id, h);
            }
            if old_plane != PLANE_LIMBO {
                multimap_remove(&mut vm.objs_by_pos, loc, T::raw_id(id));
            }
        }

        obj.area = new_area;
    }


    pub fn add_entity<H>(&mut self,
                         eid: EntityId,
                         plane: PlaneId,
                         area: SmallSet<V2>,
                         h: &mut H)
            where H: Hooks {
        self.add_viewable::<EntityTag, H>(eid, plane, area, h);
    }

    pub fn remove_entity<H>(&mut self,
                            eid: EntityId,
                            h: &mut H)
            where H: Hooks {
        self.remove_viewable::<EntityTag, H>(eid, h);
    }

    pub fn set_entity_area<H>(&mut self,
                              eid: EntityId,
                              new_plane: PlaneId,
                              new_area: SmallSet<V2>,
                              h: &mut H)
            where H: Hooks {
        self.set_viewable_area::<EntityTag, H>(eid, new_plane, new_area, h);

        let entity = &self.maps.entities.objs[eid.unwrap() as usize];
        for &cid in entity.viewers.iter() {
            h.on_entity_motion_update(cid, eid);
        }
    }

    pub fn update_entity_appearance<H>(&mut self,
                                       eid: EntityId,
                                       h: &mut H)
            where H: Hooks {
        let entity = &self.maps.entities.objs[eid.unwrap() as usize];
        for &cid in entity.viewers.iter() {
            h.on_entity_appearance_update(cid, eid);
        }
    }


    pub fn add_terrain_chunk<H>(&mut self,
                                tcid: TerrainChunkId,
                                plane: PlaneId,
                                cpos: V2,
                                h: &mut H)
            where H: Hooks {
        let mut area = SmallSet::new();
        area.insert(cpos);
        self.add_viewable::<TerrainChunkTag, H>(tcid, plane, area, h);
    }

    pub fn remove_terrain_chunk<H>(&mut self,
                                   tcid: TerrainChunkId,
                                   h: &mut H)
            where H: Hooks {
        self.remove_viewable::<TerrainChunkTag, H>(tcid, h);
    }

    pub fn update_terrain_chunk<H>(&mut self,
                                   tcid: TerrainChunkId,
                                   h: &mut H)
            where H: Hooks {
        let terrain_chunk = &self.maps.terrain_chunks.objs[tcid.unwrap() as usize];
        for &cid in terrain_chunk.viewers.iter() {
            h.on_terrain_chunk_update(cid, tcid);
        }
    }


    pub fn add_structure<H>(&mut self,
                            sid: StructureId,
                            plane: PlaneId,
                            area: SmallSet<V2>,
                            h: &mut H)
            where H: Hooks {
        self.add_viewable::<StructureTag, H>(sid, plane, area, h);
    }

    pub fn remove_structure<H>(&mut self,
                               sid: StructureId,
                               h: &mut H)
            where H: Hooks {
        self.remove_viewable::<StructureTag, H>(sid, h);
    }

    pub fn set_structure_area<H>(&mut self,
                                 sid: StructureId,
                                 new_plane: PlaneId,
                                 new_area: SmallSet<V2>,
                                 h: &mut H)
            where H: Hooks {
        self.set_viewable_area::<StructureTag, H>(sid, new_plane, new_area, h);
    }

    pub fn change_structure_template<H>(&mut self,
                                        sid: StructureId,
                                        h: &mut H)
            where H: Hooks {
        let structure = &self.maps.structures.objs[sid.unwrap() as usize];
        for &cid in structure.viewers.iter() {
            h.on_structure_template_change(cid, sid);
        }
    }


    pub fn subscribe_inventory<H>(&mut self,
                                  cid: ClientId,
                                  iid: InventoryId,
                                  h: &mut H)
            where H: Hooks {
        let viewer = unwrap_or!(self.viewers.get_mut(&(cid.unwrap() as usize)));
        let inventory_viewers = &mut self.inventory_viewers;

        viewer.visible_inventories.retain(iid, || {
            multimap_insert(inventory_viewers, iid, cid);
            h.on_inventory_appear(cid, iid);
        });
    }

    pub fn unsubscribe_inventory<H>(&mut self,
                                    cid: ClientId,
                                    iid: InventoryId,
                                    h: &mut H)
            where H: Hooks {
        let viewer = unwrap_or!(self.viewers.get_mut(&(cid.unwrap() as usize)));
        let inventory_viewers = &mut self.inventory_viewers;

        viewer.visible_inventories.release(iid, |()| {
            multimap_remove(inventory_viewers, iid, cid);
            h.on_inventory_disappear(cid, iid);
        });
    }

    pub fn update_inventory<H>(&mut self,
                               iid: InventoryId,
                               slot_idx: u8,
                               h: &mut H)
            where H: Hooks {
        let cids = unwrap_or!(self.inventory_viewers.get(&iid));
        for &cid in cids.iter() {
            h.on_inventory_update(cid, iid, slot_idx);
        }
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

