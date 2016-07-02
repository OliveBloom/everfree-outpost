use std::mem;

use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use engine::Engine;
use engine::glue::WorldFragment;
use logic;
use messages::{Messages, ClientResponse};
use physics::Physics;
use vision::Vision;
use world::{Activity, Motion, World};
use world::flags::*;
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;


engine_part2!(pub PartialEngine(world, cache, vision, messages));


/// Handler to be called just after creating an entity.
pub fn on_create(eng: &mut PartialEngine, sid: StructureId) {
    let s = eng.world.structure(sid);

    let plane = s.plane_id();
    eng.cache.update_region(&eng.world, plane, s.bounds());

    let msg_appear = logic::vision::structure_appear_message(s);
    let messages = &mut eng.messages;
    for cpos in s.bounds().reduce().div_round_signed(CHUNK_SIZE).points() {
        eng.vision.structure_add(sid, plane, cpos, |cid| {
            messages.send_client(cid, msg_appear.clone());
        });
    }
}

/// Handler to be called just before destroying an entity.
pub fn on_destroy(eng: &mut PartialEngine, sid: StructureId) {
    let s = eng.world.structure(sid);

    let plane = s.plane_id();
    eng.cache.update_region(&eng.world, plane, s.bounds());

    let msg_gone = logic::vision::structure_gone_message(s);
    let messages = &mut eng.messages;
    for cpos in s.bounds().reduce().div_round_signed(CHUNK_SIZE).points() {
        eng.vision.structure_remove(sid, plane, cpos, |cid| {
            messages.send_client(cid, msg_gone.clone());
        });
    }
}

/// Handler to be called just before destroying an entity.
pub fn on_replace(eng: &mut PartialEngine,
                  sid: StructureId,
                  old_template_id: TemplateId) {
    let s = eng.world.structure(sid);

    // Update cache
    let old_size = eng.data().structure_templates.template(old_template_id).size;
    let old_bounds = Region::sized(old_size) + s.pos();

    let new_bounds = s.bounds();

    let plane = s.plane_id();
    eng.cache.update_region(&eng.world, plane, old_bounds.join(new_bounds));

    // Notify clients
    let msg_appear = logic::vision::structure_appear_message(s);
    let msg_gone = logic::vision::structure_gone_message(s);
    let msg_replace = logic::vision::structure_replace_message(s);
    let messages = &mut eng.messages;
    let old_chunks = old_bounds.reduce().div_round_signed(CHUNK_SIZE);
    let new_chunks = new_bounds.reduce().div_round_signed(CHUNK_SIZE);

    if old_chunks != new_chunks {
        for cpos in new_chunks.points().filter(|&p| !old_chunks.contains(p)) {
            eng.vision.structure_add(sid, plane, cpos, |cid| {
                messages.send_client(cid, msg_appear.clone());
            });
        }
    } else {
        for cpos in old_chunks.points().filter(|&p| !new_chunks.contains(p)) {
            eng.vision.structure_remove(sid, plane, cpos, |cid| {
                messages.send_client(cid, msg_gone.clone());
            });
        }
    }

    eng.vision.structure_update(sid, |cid| {
        messages.send_client(cid, msg_replace.clone());
    });
}


/// Handler to be called just after importing an entity.
pub fn on_import(eng: &mut PartialEngine, sid: StructureId) {
    if eng.world.structure(sid).flags().contains(S_HAS_IMPORT_HOOK) {
        let hooks = eng.script_hooks();
        // FIXME bad transmute
        let wf: WorldFragment = unsafe { mem::transmute(eng) };
        hooks.call_structure_import_hook(wf, sid);
    }
}
