use std::collections::HashMap;

use types::*;

use world::{World, Plane};
use world::extra::Extra;
use world::ops::{self, OpResult};


pub fn create(w: &mut World, name: String) -> OpResult<PlaneId> {
    let p = Plane {
        name: name,

        loaded_chunks: HashMap::new(),
        saved_chunks: HashMap::new(),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,

        version: w.snapshot.version() + 1,
    };

    let pid = unwrap!(w.planes.insert(p));
    post_init(w, pid);
    Ok(pid)
}

pub fn create_unchecked(w: &mut World) -> PlaneId {
    let pid = w.planes.insert(Plane {
        name: String::new(),

        loaded_chunks: HashMap::new(),
        saved_chunks: HashMap::new(),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,

        version: w.snapshot.version() + 1,
    }).unwrap();     // Shouldn't fail when stable_id == NO_STABLE_ID
    pid
}

pub fn post_init(w: &mut World,
                 pid: PlaneId) {
    trace!("post_init({:?})", pid);
    let stable_pid = w.planes.pin(pid);

    // limbo_entities is manipulated using multimap_insert/multimap_remove, so if there are no
    // entities in limbo targeting this plane, there may be not HashSet under the given key.
    if let Some(eids) = w.limbo_entities.remove(&stable_pid) {
        let mut eids_vec = Vec::with_capacity(eids.len());
        for &eid in eids.iter() {
            w.entities[eid].plane = pid;
            eids_vec.push(eid);
        }
        w.entities_by_plane.insert(pid, eids);

        trace!("post_init: transfer {} entities for {:?} ({:?})", eids_vec.len(), pid, stable_pid);
        trace!("post_init: entities: {:?}", eids_vec);
    }
}

pub fn pre_fini(w: &mut World,
                pid: PlaneId) {
    trace!("pre_fini({:?})", pid);
    let stable_pid = w.planes.pin(pid);

    // Same multimap_* stuff as is post_init.
    if let Some(eids) = w.entities_by_plane.remove(&pid) {
        let mut eids_vec = Vec::with_capacity(eids.len());
        for &eid in eids.iter() {
            w.entities[eid].plane = PLANE_LIMBO;
            eids_vec.push(eid);
        }
        w.limbo_entities.insert(stable_pid, eids);
    }
}

pub fn destroy(w: &mut World,
               pid: PlaneId) -> OpResult<()> {
    pre_fini(w, pid);
    let p = unwrap!(w.planes.remove(pid));
    w.snapshot.record_plane(pid, &p);

    for &tcid in p.loaded_chunks.values() {
        ops::terrain_chunk::destroy(w, tcid).unwrap();
    }

    Ok(())
}
