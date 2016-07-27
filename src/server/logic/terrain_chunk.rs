use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};
use util::SmallVec;

use engine::Engine;
use engine::split2::Coded;
use logic;
use messages::{Messages, ClientResponse};
use physics::Physics;
use vision::Vision;
use world::{Activity, Motion, World};
use world::object::*;


engine_part2!(pub PartialEngine(world, cache, vision, messages));


/// Handler to be called just after creating a terrain chunk.
pub fn on_create(eng: &mut PartialEngine, tcid: TerrainChunkId) {
    let tc = eng.world.terrain_chunk(tcid);

    let data = eng.data();
    let plane = tc.plane_id();
    let cpos = tc.chunk_pos();
    eng.cache.add_chunk(data, plane, cpos, tc.blocks());

    let msg = logic::vision::terrain_chunk_message(tc);
    let messages = &mut eng.messages;
    eng.vision.terrain_chunk_add(tcid, plane, cpos, |cid| {
        messages.send_client(cid, msg.clone());
    });
}

/// Handler to be called just before destroying a terrain chunk.
pub fn on_destroy(eng: &mut PartialEngine, tcid: TerrainChunkId) {
    let tc = eng.world.terrain_chunk(tcid);

    let plane = tc.plane_id();
    let cpos = tc.chunk_pos();
    eng.cache.remove_chunk(plane, cpos);

    // No message on chunk destroy
    eng.vision.terrain_chunk_remove(tcid, plane, cpos, |_| {
    });
}

/// Similar to `on_destroy`, but also invokes `on_destroy` for all child objects.
pub fn on_destroy_recursive(eng: &mut logic::world::EngineLifecycle, tcid: TerrainChunkId) {
    on_destroy(eng.refine(), tcid);
    let mut v = SmallVec::new();
    for s in eng.world.terrain_chunk(tcid).child_structures() {
        v.push(s.id());
    }
    for &sid in v.iter() {
        logic::structure::on_destroy_recursive(eng, sid);
    }
}

/// Handler to be called just after modifying a terrain chunk.
pub fn on_update(eng: &mut PartialEngine, tcid: TerrainChunkId) {
    let tc = eng.world.terrain_chunk(tcid);

    let data = eng.data();
    let plane = tc.plane_id();
    let cpos = tc.chunk_pos();
    eng.cache.update_chunk(data, plane, cpos, tc.blocks());

    let msg = logic::vision::terrain_chunk_message(tc);
    let messages = &mut eng.messages;
    eng.vision.terrain_chunk_update(tcid, |cid| {
        messages.send_client(cid, msg.clone());
    });
}
