use types::*;

use engine::Engine;
use engine::split2::Coded;
use logic;
use world::flags::I_HAS_CHANGE_HOOK;
use world::object::*;


engine_part2!(pub EngineLifecycle(world, vision, messages, dialogs));

pub fn on_destroy(eng: &mut EngineLifecycle, iid: InventoryId) {
    logic::dialogs::clear_inventory_users(eng.refine(), iid);
}

pub fn on_destroy_recursive(eng: &mut logic::world::EngineLifecycle, iid: InventoryId) {
    on_destroy(eng.refine(), iid);
}


engine_part2!(pub EngineSubscribe(world, vision, messages));

pub fn subscribe(eng: &mut EngineSubscribe, cid: ClientId, iid: InventoryId) {
    info!("subscribe {:?} to {:?}", cid, iid);
    let world = &mut eng.world;
    let messages = &mut eng.messages;
    eng.vision.subscribe_inventory(cid, iid, || {
        let i = world.inventory(iid);
        let msg = logic::vision::inventory_appear_message(i);
        messages.send_client(cid, msg);
    });
}

pub fn unsubscribe(eng: &mut EngineSubscribe, cid: ClientId, iid: InventoryId) {
    info!("unsubscribe {:?} from {:?}", cid, iid);
    let world = &mut eng.world;
    let messages = &mut eng.messages;
    eng.vision.unsubscribe_inventory(cid, iid, || {
        let i = world.inventory(iid);
        let msg = logic::vision::inventory_gone_message(i);
        messages.send_client(cid, msg);
    });
}

pub fn on_update(eng: &mut EngineSubscribe, iid: InventoryId, slot_idx: u8) {
    let i = eng.world.inventory(iid);
    let msg = logic::vision::inventory_update_message(i, slot_idx);

    let messages = &mut eng.messages;
    eng.vision.update_inventory(iid, |cid| {
        messages.send_client(cid, msg.clone());
    });
}


pub fn call_update_hook(eng: &mut Engine, iid: InventoryId) {
    let flags = eng.world.inventory(iid).flags();
    if flags.contains(I_HAS_CHANGE_HOOK) {
        warn_on_err!(eng.script_hooks.call_inventory_change_hook(eng, iid));
    }
}
