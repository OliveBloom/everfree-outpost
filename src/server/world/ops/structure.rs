use std::collections::{HashMap, HashSet};

use libphysics::CHUNK_SIZE;
use types::*;
use util::{multimap_insert, multimap_remove};

use world::{World, Structure, StructureAttachment, StructureFlags};
use world::extra::Extra;
use world::ops::{self, OpResult};


pub fn create(w: &mut World,
              pid: PlaneId,
              pos: V3,
              tid: TemplateId) -> OpResult<StructureId> {
    let t = unwrap!(w.data.get_template(tid));
    let bounds = Region::new(pos, pos + t.size);

    if bounds.min.z < 0 || bounds.max.z > CHUNK_SIZE {
        fail!("structure placement blocked by map bounds");
    }

    let stable_pid = w.planes.pin(pid);
    let s = Structure {
        stable_plane: stable_pid,
        plane: pid,
        pos: pos,
        template: tid,

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: StructureFlags::empty(),
        attachment: StructureAttachment::Plane,
        child_inventories: HashSet::new(),

        version: w.snapshot.version() + 1,
    };

    let sid = unwrap!(w.structures.insert(s));
    add_to_lookup(&mut w.structures_by_chunk, sid, pid, bounds);
    Ok(sid)
}

pub fn create_unchecked(w: &mut World) -> StructureId {
    let sid = w.structures.insert(Structure {
        stable_plane: Stable::new(0),
        plane: PlaneId(0),
        pos: scalar(0),
        template: 0,

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: StructureFlags::empty(),
        attachment: StructureAttachment::Plane,
        child_inventories: HashSet::new(),

        version: w.snapshot.version() + 1,
    }).unwrap();     // Shouldn't fail when stable_id == NO_STABLE_ID
    sid
}

pub fn post_init(w: &mut World,
                 sid: StructureId) {
    let (pid, bounds) = {
        let s = &w.structures[sid];
        let t = w.data.template(s.template);

        (s.plane, Region::new(s.pos, s.pos + t.size))
    };

    add_to_lookup(&mut w.structures_by_chunk, sid, pid, bounds);
}

pub fn pre_fini(w: &mut World,
                sid: StructureId) {
    let (pid, bounds) = {
        let s = &w.structures[sid];
        let t = w.data.template(s.template);

        (s.plane, Region::new(s.pos, s.pos + t.size))
    };

    remove_from_lookup(&mut w.structures_by_chunk, sid, pid, bounds);
}

pub fn destroy(w: &mut World,
               sid: StructureId) -> OpResult<()> {
    use world::StructureAttachment::*;
    let s = unwrap!(w.structures.remove(sid));
    w.snapshot.record_structure(sid, &s);

    let t = w.data.template(s.template);
    let bounds = Region::new(s.pos, s.pos + t.size);
    remove_from_lookup(&mut w.structures_by_chunk, sid, s.plane, bounds);

    match s.attachment {
        // TODO: proper support for Plane attachment
        Plane => {},
        Chunk => {
            // Plane or chunk may not be loaded, since destruction proceeds top-down.
            if let Some(p) = w.planes.get_mut(s.plane) {
                let chunk_pos = s.pos.reduce().div_floor(scalar(CHUNK_SIZE));
                if let Some(&tcid) = p.loaded_chunks.get(&chunk_pos) {
                    if let Some(tc) = w.terrain_chunks.get_mut(tcid) {
                        tc.child_structures.remove(&sid);
                    }
                }
            }
        },
    }

    for &iid in s.child_inventories.iter() {
        ops::inventory::destroy(w, iid).unwrap();
    }

    Ok(())
}

pub fn attach(w: &mut World,
              sid: StructureId,
              new_attach: StructureAttachment) -> OpResult<StructureAttachment> {
    use world::StructureAttachment::*;

    let s = unwrap!(w.structures.get_mut(sid));
    let old_attach = s.attachment;

    if new_attach == old_attach {
        return Ok(new_attach);
    }

    let chunk_pos = s.pos().reduce().div_floor(scalar(CHUNK_SIZE));

    match new_attach {
        // TODO: proper support for Plane attachment
        Plane => {},
        Chunk => {
            // Structures can exist only in planes that are currently loaded.
            let p = &w.planes[s.plane];
            let &tcid = unwrap!(p.loaded_chunks.get(&chunk_pos),
                                "can't attach structure to unloaded chunk");
            let tc = &mut w.terrain_chunks[tcid];
            tc.child_structures.insert(sid);
            // No more checks beyond this point.
        },
    }

    match old_attach {
        Plane => {},
        Chunk => {
            // If we're detaching from Chunk, we know the containing chunk is loaded because `c` is
            // loaded and has attachment Chunk.
            let p = &w.planes[s.plane];
            let tcid = p.loaded_chunks[&chunk_pos];
            let tc = &mut w.terrain_chunks[tcid];
            tc.child_structures.remove(&sid);
        },
    }

    s.attachment = new_attach;
    Ok(old_attach)
}

pub fn replace(w: &mut World,
               sid: StructureId,
               new_tid: TemplateId) -> OpResult<()> {
    let bounds = {
        let s = unwrap!(w.structures.get(sid));
        let t = unwrap!(w.data.get_template(new_tid));
        Region::new(s.pos, s.pos + t.size)
    };

    // If the structure is changing size, make sure it's still within bounds.
    if bounds.min.z < 0 || bounds.max.z > CHUNK_SIZE {
        fail!("structure replacement blocked by map bounds");
    }

    w.structures[sid].template = new_tid;

    Ok(())
}

fn add_to_lookup(lookup: &mut HashMap<(PlaneId, V2), HashSet<StructureId>>,
                 sid: StructureId,
                 pid: PlaneId,
                 bounds: Region) {
    let chunk_bounds = bounds.reduce().div_round_signed(CHUNK_SIZE);
    for chunk_pos in chunk_bounds.points() {
        multimap_insert(lookup, (pid, chunk_pos), sid);
    }
}

fn remove_from_lookup(lookup: &mut HashMap<(PlaneId, V2), HashSet<StructureId>>,
                      sid: StructureId,
                      pid: PlaneId,
                      bounds: Region) {
    let chunk_bounds = bounds.reduce().div_round_signed(CHUNK_SIZE);
    for chunk_pos in chunk_bounds.points() {
        multimap_remove(lookup, (pid, chunk_pos), sid);
    }
}
