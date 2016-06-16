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


engine_part2!(pub PartialEngine(world, vision, messages));

pub fn subscribe(eng: &mut PartialEngine, cid: ClientId, iid: InventoryId) {
    let world = &mut eng.world;
    let messages = &mut eng.messages;
    eng.vision.subscribe_inventory(cid, iid, || {
        let i = world.inventory(iid);
        let msg = logic::vision::inventory_appear_message(i);
        messages.send_client(cid, msg);
    });
}

pub fn unsubscribe(eng: &mut PartialEngine, cid: ClientId, iid: InventoryId) {
    let world = &mut eng.world;
    let messages = &mut eng.messages;
    eng.vision.unsubscribe_inventory(cid, iid, || {
        let i = world.inventory(iid);
        let msg = logic::vision::inventory_gone_message(i);
        messages.send_client(cid, msg);
    });
}

pub fn on_update(eng: &mut PartialEngine, iid: InventoryId, slot_idx: u8) {
    let i = eng.world.inventory(iid);
    let msg = logic::vision::inventory_update_message(i, slot_idx);

    let messages = &mut eng.messages;
    eng.vision.update_inventory(iid, |cid| {
        messages.send_client(cid, msg.clone());
    });
}
