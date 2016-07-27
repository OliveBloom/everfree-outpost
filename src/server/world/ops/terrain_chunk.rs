use std::collections::HashSet;

use types::*;

use world::{World, TerrainChunk, TerrainChunkFlags};
use world::flags;
use world::extra::Extra;
use world::ops::{self, OpResult};


pub fn create(w: &mut World,
              pid: PlaneId,
              cpos: V2) -> OpResult<TerrainChunkId> {
    let stable_pid = w.planes.pin(pid);
    let tc = TerrainChunk {
        stable_plane: stable_pid,
        plane: pid,
        cpos: cpos,
        blocks: Box::new(PLACEHOLDER_CHUNK),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: flags::TC_GENERATION_PENDING,
        child_structures: HashSet::new(),

        version: w.snapshot.version() + 1,
    };

    // unwrap() always succeeds because stable_id is NO_STABLE_ID.
    let tcid = w.terrain_chunks.insert(tc).unwrap();
    post_init(w, tcid);
    Ok(tcid)
}

pub fn create_unchecked(w: &mut World) -> TerrainChunkId {
    let tcid = w.terrain_chunks.insert(TerrainChunk {
        stable_plane: Stable::new(0),
        plane: PlaneId(0),
        cpos: scalar(0),
        blocks: Box::new(EMPTY_CHUNK),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: TerrainChunkFlags::empty(),
        child_structures: HashSet::new(),

        version: w.snapshot.version() + 1,
    }).unwrap();     // Shouldn't fail when stable_id == NO_STABLE_ID
    tcid
}

pub fn post_init(w: &mut World,
                 tcid: TerrainChunkId) {
    let tc = &w.terrain_chunks[tcid];
    // TODO: error handling: check for duplicate entries with same cpos
    w.planes[tc.plane].loaded_chunks.insert(tc.cpos, tcid);
}

pub fn pre_fini(w: &mut World,
                tcid: TerrainChunkId) {
    let tc = &w.terrain_chunks[tcid];
    // Containing plane may be missing during recursive destruction.
    w.planes.get_mut(tc.plane)
     .map(|p| p.loaded_chunks.remove(&tc.cpos));
}

pub fn destroy(w: &mut World,
               tcid: TerrainChunkId) -> OpResult<()> {
    trace!("destroy {:?}", tcid);
    pre_fini(w, tcid);
    let tc = unwrap!(w.terrain_chunks.remove(tcid));
    w.snapshot.record_terrain_chunk(tcid, &tc);

    for &sid in tc.child_structures.iter() {
        ops::structure::destroy(w, sid).unwrap();
    }

    Ok(())
}
