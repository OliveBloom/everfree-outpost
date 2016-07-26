use std::collections::HashSet;

use types::*;

use world::{TerrainChunk, TerrainChunkFlags};
use world::Fragment;
use world::flags;
use world::extra::Extra;
use world::ops::{self, OpResult};


pub fn create<'d, F>(f: &mut F,
                     pid: PlaneId,
                     cpos: V2) -> OpResult<TerrainChunkId>
        where F: Fragment<'d> {
    let stable_pid = f.world_mut().planes.pin(pid);
    let tc = TerrainChunk {
        stable_plane: stable_pid,
        plane: pid,
        cpos: cpos,
        blocks: Box::new(PLACEHOLDER_CHUNK),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: flags::TC_GENERATION_PENDING,
        child_structures: HashSet::new(),

        version: f.world().snapshot.version() + 1,
    };

    // unwrap() always succeeds because stable_id is NO_STABLE_ID.
    let tcid = f.world_mut().terrain_chunks.insert(tc).unwrap();
    post_init(f, tcid);
    Ok(tcid)
}

pub fn create_unchecked<'d, F>(f: &mut F) -> TerrainChunkId
        where F: Fragment<'d> {
    let w = f.world_mut();
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

pub fn post_init<'d, F>(f: &mut F,
                        tcid: TerrainChunkId)
        where F: Fragment<'d> {
    let w = f.world_mut();
    let tc = &w.terrain_chunks[tcid];
    // TODO: error handling: check for duplicate entries with same cpos
    w.planes[tc.plane].loaded_chunks.insert(tc.cpos, tcid);
}

pub fn pre_fini<'d, F>(f: &mut F,
                       tcid: TerrainChunkId)
        where F: Fragment<'d> {
    let w = f.world_mut();
    let tc = &w.terrain_chunks[tcid];
    // Containing plane may be missing during recursive destruction.
    w.planes.get_mut(tc.plane)
     .map(|p| p.loaded_chunks.remove(&tc.cpos));
}

pub fn destroy<'d, F>(f: &mut F,
                      tcid: TerrainChunkId) -> OpResult<()>
        where F: Fragment<'d> {
    trace!("destroy {:?}", tcid);
    pre_fini(f, tcid);
    let tc = unwrap!(f.world_mut().terrain_chunks.remove(tcid));
    f.world_mut().snapshot.record_terrain_chunk(tcid, &tc);

    for &sid in tc.child_structures.iter() {
        ops::structure::destroy(f, sid).unwrap();
    }

    Ok(())
}
