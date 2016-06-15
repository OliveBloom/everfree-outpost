use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use engine::Engine;
use logic;
use messages::{Messages, ClientResponse};
use physics::Physics;
use vision::Vision;
use world::{Activity, Motion, World};
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;


engine_part2!(pub PartialEngine(world, cache, vision, messages));


/// Handler to be called just after creating an entity.
pub fn on_create(eng: &mut PartialEngine, tcid: TerrainChunkId) {
    let tc = eng.world.terrain_chunk(tcid);

    let plane = tc.plane_id();
    let cpos = tc.chunk_pos();
    eng.cache.add_chunk(&eng.world, plane, cpos);

    let msg = logic::vision::terrain_chunk_message(tc);
    let messages = &mut eng.messages;
    eng.vision.terrain_chunk_add(tcid, plane, cpos, |cid| {
        messages.send_client(cid, msg.clone());
    });
}

/// Handler to be called just before destroying an entity.
pub fn on_destroy(eng: &mut PartialEngine, tcid: TerrainChunkId) {
    let tc = eng.world.terrain_chunk(tcid);

    let plane = tc.plane_id();
    let cpos = tc.chunk_pos();
    eng.cache.remove_chunk(plane, cpos);

    // No message on chunk destroy
    eng.vision.terrain_chunk_remove(tcid, plane, cpos, |_| {
    });
}

/// Handler to be called just before destroying an entity.
pub fn on_update(eng: &mut PartialEngine, tcid: TerrainChunkId) {
    let tc = eng.world.terrain_chunk(tcid);

    let plane = tc.plane_id();
    eng.cache.update_region(&eng.world, plane, tc.bounds());

    let msg = logic::vision::terrain_chunk_message(tc);
    let messages = &mut eng.messages;
    eng.vision.terrain_chunk_update(tcid, |cid| {
        messages.send_client(cid, msg.clone());
    });
}
