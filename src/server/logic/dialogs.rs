use types::*;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use dialogs::TargetId;
use engine::Engine;
use engine::split2::Coded;
use input::{Action, InputBits, INPUT_DIR_MASK};
use logic;
use messages::ClientResponse;
use physics::UpdateKind;
use timing::next_tick;
use world::Activity;
use world::object::*;


engine_part2!(pub OnlyDialogs(dialogs));

pub fn close_dialog(eng: &mut Engine, cid: ClientId) {
    let (eng, only_dialogs) = eng.split();
    let &mut OnlyDialogs { ref mut dialogs, .. } = only_dialogs;
    dialogs.clear_dialog(cid, |target| match target {
        TargetId::Inventory(iid) => logic::inventory::unsubscribe(eng, cid, iid),
        TargetId::Structure(_) => {},
    });
}
