use types::*;

use engine::split::{EngineRef, Open};
use input::{InputBits};
use messages::ClientResponse;
use msg::ExtraArg;
use physics;
use world::object::*;
use vision;


pub fn input(mut eng: EngineRef, cid: ClientId, input: InputBits) {
    let now = eng.now();

    let target_velocity = input.to_velocity();
    if let Some(eid) = eng.world().get_client(cid).and_then(|c| c.pawn_id()) {
        warn_on_err!(physics::Fragment::set_velocity(
                &mut eng.as_physics_fragment(), now, eid, target_velocity));
    }
}

pub fn interact(eng: EngineRef, cid: ClientId, args: Option<ExtraArg>) {
    warn_on_err!(eng.script_hooks().call_client_interact(eng, cid, args));
}

pub fn use_item(eng: EngineRef, cid: ClientId, item_id: ItemId, args: Option<ExtraArg>) {
    warn_on_err!(eng.script_hooks().call_client_use_item(eng, cid, item_id, args));
}

pub fn use_ability(eng: EngineRef, cid: ClientId, item_id: ItemId, args: Option<ExtraArg>) {
    warn_on_err!(eng.script_hooks().call_client_use_ability(eng, cid, item_id, args));
}

pub fn open_inventory(_eng: EngineRef, cid: ClientId) {
    error!("UNIMPLEMENTED: open_inventory - called by {:?}", cid);
}

pub fn unsubscribe_inventory(mut eng: EngineRef, cid: ClientId, iid: InventoryId) {
    // No need for cherks - unsubscribe_inventory does nothing if the arguments are invalid.
    vision::Fragment::unsubscribe_inventory(&mut eng.as_vision_fragment(), cid, iid);
}

pub fn chat(mut eng: EngineRef, cid: ClientId, msg: String) {
    if msg.starts_with("/") && !msg.starts_with("/l ") {
        warn_on_err!(eng.script_hooks().call_client_chat_command(eng, cid, &msg));
        return;
    }

    let (msg, local) =
        if msg.starts_with("/l ") { (&msg[3..], true) }
        else { (&msg as &str, false) };
    if msg.len() > 400 {
        warn!("{:?}: bad request: chat message too long ({})", cid, msg.len());
        return;
    }

    let Open { world, messages, chat, .. } = eng.open();
    if !local {
        chat.send_global(world, messages, cid, msg);
    } else {
        chat.send_local(world, messages, cid, msg);
    }
}
