use types::*;
use libphysics::CHUNK_SIZE;
use util::SmallVec;

use cache::TerrainCache;
use data::StructureTemplate;
use engine::split2::Coded;
use logic;
use world::flags::*;
use world::object::*;
use world::OpResult;


engine_part2!(pub EngineLifecycle(world, cache, vision, messages, dialogs, timer));

/// Handler to be called just after creating a structure.
pub fn on_create(eng: &mut EngineLifecycle, sid: StructureId) {
    let s = eng.world.structure(sid);

    let plane = s.plane_id();
    eng.cache.add_structure(plane, s.pos(), s.template());

    let msg_appear = logic::vision::structure_appear_message(s);
    let messages = &mut eng.messages;
    for cpos in s.bounds().reduce().div_round_signed(CHUNK_SIZE).points() {
        eng.vision.structure_add(sid, plane, cpos, |cid| {
            messages.send_client(cid, msg_appear.clone());
        });
    }
}

/// Handler to be called just before destroying a structure.
pub fn on_destroy(eng: &mut EngineLifecycle, sid: StructureId) {
    logic::dialogs::clear_structure_users(eng.refine(), sid);

    let s = eng.world.structure(sid);

    let plane = s.plane_id();
    eng.cache.remove_structure(plane, s.pos(), s.template());

    let msg_gone = logic::vision::structure_gone_message(s);
    let messages = &mut eng.messages;
    for cpos in s.bounds().reduce().div_round_signed(CHUNK_SIZE).points() {
        eng.vision.structure_remove(sid, plane, cpos, |cid| {
            messages.send_client(cid, msg_gone.clone());
        });
    }
}

/// Similar to `on_destroy`, but also invokes `on_destroy` for all child objects.
pub fn on_destroy_recursive(eng: &mut logic::world::EngineLifecycle, sid: StructureId) {
    on_destroy(eng.refine(), sid);
    let mut v = SmallVec::new();
    for i in eng.world.structure(sid).child_inventories() {
        v.push(i.id());
    }
    for &iid in v.iter() {
        logic::inventory::on_destroy_recursive(eng, iid);
    }
}

/// Handler to be called just after changing a structure's template.
pub fn on_replace(eng: &mut EngineLifecycle,
                  sid: StructureId,
                  old_template_id: TemplateId) {
    let s = eng.world.structure(sid);

    // Update cache
    let old_template = eng.data().structure_templates.template(old_template_id);
    let old_bounds = Region::sized(old_template.size) + s.pos();

    let new_bounds = s.bounds();

    let plane = s.plane_id();
    eng.cache.remove_structure(plane, s.pos(), old_template);
    eng.cache.add_structure(plane, s.pos(), s.template());

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
pub fn on_import(eng: &mut EngineLifecycle, sid: StructureId) {
    if eng.world.structure(sid).flags().contains(S_HAS_IMPORT_HOOK) {
        warn_on_err!(eng.script_hooks().call_structure_import_hook(eng, sid));
    }
}



engine_part2!(pub EngineCheck(world, cache));

pub fn checked_create(eng: &mut EngineCheck,
                      pid: PlaneId,
                      pos: V3,
                      template_id: TemplateId) -> OpResult<StructureId> {
    let template = unwrap!(eng.data().structure_templates.get_template(template_id));
    try!(check_placement(&eng.cache, pid, pos, template));
    let s = try!(eng.world.create_structure(pid, pos, template_id));
    Ok(s.id())
}

pub fn checked_replace(eng: &mut EngineCheck,
                       sid: StructureId,
                       template_id: TemplateId) -> OpResult<()> {
    let new_template = unwrap!(eng.data().structure_templates.get_template(template_id));

    let mut s = unwrap!(eng.world.get_structure_mut(sid));
    let old_template = s.template();

    try!(check_replacement(&mut eng.cache, s.plane_id(), s.pos(), old_template, new_template));
    try!(s.set_template_id(template_id));
    Ok(())
}

fn check_placement(cache: &TerrainCache,
                   pid: PlaneId,
                   pos: V3,
                   template: &StructureTemplate) -> OpResult<()> {
    let bounds = Region::sized(template.size) + pos;

    for pos in bounds.points() {
        let flags = template.shape[bounds.index(pos)];
        if !flags.occupied() {
            continue;
        }

        let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE));
        let base = (cpos * scalar(CHUNK_SIZE)).extend(0);
        let entry = unwrap!(cache.get(pid, cpos));
        // Cell not present -> it's empty, anything can overlap with it.
        let cell = unwrap_or!(entry.get_cell(pos - base), continue);

        if cell.layers[template.layer as usize].occupied() {
            fail!("structure blocked (layering)");
        }

        let overlap_parts = cell.computed.parts() & flags.parts();
        if !overlap_parts.is_empty() {
            fail!("structure blocked (parts)");
        }
    }

    Ok(())
}

fn check_replacement(cache: &mut TerrainCache,
                     pid: PlaneId,
                     pos: V3,
                     old_template: &StructureTemplate,
                     new_template: &StructureTemplate) -> OpResult<()> {

    // If the new template sets strictly fewer flags than the old one, then placement is guaranteed
    // to succeed.
    if Region::sized(old_template.size).contains_inclusive(new_template.size) {
        let bounds = Region::sized(new_template.size);
        let mut ok = true;
        for pos in bounds.points() {
            let idx = bounds.index(pos);
            let old = old_template.shape[idx];
            let new = new_template.shape[idx];
            if old != new && (!old.contains(new) || old.shape() != new.shape()) {
                ok = false;
                break;
            }
        }
        if ok {
            return Ok(());
        }
    }

    // Main placement check.  Remove the old structure, and check if the new one fits.
    cache.remove_structure(pid, pos, old_template);
    let result = check_placement(cache, pid, pos, new_template);
    // Always put the old template back.  Let the future `on_replace` make the actual change.
    cache.add_structure(pid, pos, old_template);
    result
}


