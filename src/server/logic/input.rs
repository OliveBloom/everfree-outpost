use types::*;
use libcommon_proto::ExtraArg;

use engine::Engine;


pub fn interact(eng: &mut Engine, cid: ClientId, args: Option<ExtraArg>) {
    warn_on_err!(eng.script_hooks.call_client_interact(eng, cid, args));
}

pub fn use_item(eng: &mut Engine, cid: ClientId, item_id: ItemId, args: Option<ExtraArg>) {
    warn_on_err!(eng.script_hooks.call_client_use_item(eng, cid, item_id, args));
}

pub fn use_ability(eng: &mut Engine, cid: ClientId, item_id: ItemId, args: Option<ExtraArg>) {
    warn_on_err!(eng.script_hooks.call_client_use_ability(eng, cid, item_id, args));
}

pub fn open_inventory(_eng: &mut Engine, cid: ClientId) {
    error!("UNIMPLEMENTED: open_inventory - called by {:?}", cid);
}

pub fn chat(eng: &mut Engine, cid: ClientId, msg: String) {
    if msg.starts_with("/") && !msg.starts_with("/l ") {
        warn_on_err!(eng.script_hooks.call_client_chat_command(eng, cid, &msg));
        return;
    }

    let (msg, local) =
        if msg.starts_with("/l ") { (&msg[3..], true) }
        else { (&msg as &str, false) };
    if msg.len() > 400 {
        warn!("{:?}: bad request: chat message too long ({})", cid, msg.len());
        return;
    }

    if !local {
        // TODO: use an engine_part!()?
        eng.chat.send_global(&eng.world, &mut eng.messages, cid, msg);
    } else {
        eng.chat.send_local(&eng.world, &mut eng.messages, cid, msg);
    }
}
