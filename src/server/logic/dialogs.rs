use types::*;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use dialogs::{DialogType, TargetId};
use engine::Engine;
use engine::split2::Coded;
use input::{Action, InputBits, INPUT_DIR_MASK};
use logic;
use messages::ClientResponse;
use physics::UpdateKind;
use timing::next_tick;
use world::Activity;
use world::object::*;


engine_part2!(pub EngineDialogs(world, vision, messages, dialogs));
engine_part2!(pub EngineSubscribe(world, vision, messages));
engine_part2!(pub OnlyDialogs(dialogs));

pub fn open_dialog(eng: &mut EngineDialogs, cid: ClientId, dialog: DialogType) {
    let (eng, only_dialogs): (&mut EngineSubscribe, &mut OnlyDialogs) = eng.split();
    let &mut OnlyDialogs { ref mut dialogs, .. } = only_dialogs;

    dialogs.set_dialog(cid, dialog, |target, added| {
        match target {
            TargetId::Inventory(iid) =>
                if added { logic::inventory::subscribe(eng.refine(), cid, iid) }
                else { logic::inventory::unsubscribe(eng.refine(), cid, iid) },
            TargetId::Structure(_) => {},
        }
    });
}

pub fn close_dialog(eng: &mut EngineDialogs, cid: ClientId) {
    let (eng, only_dialogs): (&mut EngineSubscribe, &mut OnlyDialogs) = eng.split();
    let &mut OnlyDialogs { ref mut dialogs, .. } = only_dialogs;
    dialogs.clear_dialog(cid, |target| match target {
        TargetId::Inventory(iid) => logic::inventory::unsubscribe(eng.refine(), cid, iid),
        TargetId::Structure(_) => {},
    });
}

pub fn clear_inventory_users(eng: &mut EngineDialogs, iid: InventoryId) {
    clear_users(eng, TargetId::Inventory(iid));
}

pub fn clear_structure_users(eng: &mut EngineDialogs, sid: StructureId) {
    clear_users(eng, TargetId::Structure(sid));
}

pub fn clear_users(eng: &mut EngineDialogs, target: TargetId) {
    let (eng, only_dialogs): (&mut EngineSubscribe, &mut OnlyDialogs) = eng.split();
    let &mut OnlyDialogs { ref mut dialogs, .. } = only_dialogs;
    dialogs.clear_users(target, |cid, target| match target {
        None => eng.messages.send_client(cid, ClientResponse::CancelDialog),
        Some(TargetId::Inventory(iid)) =>
            logic::inventory::unsubscribe(eng.refine(), cid, iid),
        Some(TargetId::Structure(sid)) => {},
    });
}
