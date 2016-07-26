use std::cmp;
use std::u8;

use types::*;
use util::SmallVec;
use util::StrResult;

use dialogs::{DialogType, TargetId};
use engine::Engine;
use engine::split::EngineRef;
use engine::split2::Coded;
use logic;
use messages::{ClientResponse, Dialog};
use world;
use world::Item;
use world::fragment::{Fragment, DummyFragment};
use world::object::*;
use vision;


pub fn open_inventory(eng: &mut Engine, cid: ClientId, iid: InventoryId) -> StrResult<()> {
    // Check that IDs are valid.
    unwrap!(eng.world.get_client(cid));
    unwrap!(eng.world.get_inventory(iid));

    logic::inventory::subscribe(eng.refine(), cid, iid);
    eng.messages.send_client(cid, ClientResponse::OpenDialog(Dialog::Inventory(iid)));

    Ok(())
}

pub fn open_container(eng: &mut Engine,
                      cid: ClientId,
                      iid1: InventoryId,
                      iid2: InventoryId) -> StrResult<()> {
    use logic::dialogs::OnlyDialogs;

    // Check that IDs are valid.
    unwrap!(eng.world.get_client(cid));
    unwrap!(eng.world.get_inventory(iid1));
    unwrap!(eng.world.get_inventory(iid2));

    logic::dialogs::open_dialog(eng.refine(), cid, DialogType::Container(iid1, iid2));
    eng.messages.send_client(cid, ClientResponse::OpenDialog(Dialog::Container(iid1, iid2)));
    Ok(())
}

pub fn open_crafting(eng: &mut Engine,
                     cid: ClientId,
                     sid: StructureId,
                     iid: InventoryId) -> StrResult<()> {
    use logic::dialogs::OnlyDialogs;

    // Check that IDs are valid.
    unwrap!(eng.world.get_client(cid));
    unwrap!(eng.world.get_inventory(iid));

    let template_id = {
        let s = unwrap!(eng.world.get_structure(sid));
        s.template_id()
    };

    let (eng, only_dialogs) = eng.split();
    let &mut OnlyDialogs { ref mut dialogs, .. } = only_dialogs;

    dialogs.set_dialog(cid, DialogType::Crafting(sid, iid), |target, added| {
        match target {
            TargetId::Inventory(iid) =>
                if added { logic::inventory::subscribe(eng, cid, iid) }
                else { logic::inventory::unsubscribe(eng, cid, iid) },
            TargetId::Structure(_) => {},
        }
    });
    let dialog = Dialog::Crafting(template_id, sid, iid);
    eng.messages.send_client(cid, ClientResponse::OpenDialog(dialog));

    Ok(())
}

pub fn set_main_inventories(eng: &mut Engine,
                            cid: ClientId,
                            item_iid: InventoryId,
                            ability_iid: InventoryId) -> StrResult<()> {
    // Check that IDs are valid.
    unwrap!(eng.world.get_client(cid));
    unwrap!(eng.world.get_inventory(item_iid));
    unwrap!(eng.world.get_inventory(ability_iid));

    logic::inventory::subscribe(eng.refine(), cid, item_iid);
    logic::inventory::subscribe(eng.refine(), cid, ability_iid);
    eng.messages.send_client(cid, ClientResponse::MainInventory(item_iid));
    eng.messages.send_client(cid, ClientResponse::AbilityInventory(ability_iid));

    Ok(())
}


pub fn move_items2(eng: &mut Engine,
                   from_iid: InventoryId,
                   from_slot: u8,
                   to_iid: InventoryId,
                   to_slot: u8,
                   count: u8) -> StrResult<u8> {

    let src = unwrap!(eng.world.get_inventory(from_iid)
        .and_then(|i| i.contents().get(from_slot as usize).map(|&slot| slot)));
    let to_move = cmp::min(src.count, count);
    let mut remaining = to_move;

    // Update destination, keeping track of which slots were updated and how much was moved.
    let mut updated_slots = SmallVec::new();
    {
        let mut wf = DummyFragment::new(&mut eng.world);
        let mut i = unwrap!(wf.get_inventory_mut(to_iid));
        if to_slot != NO_SLOT && to_slot as usize >= i.contents().len() {
            fail!("bad slot for inventory");
        }
        // Cannot fail past this point.

        let mut move_into = |slot: &mut Item, idx: u8| -> bool {
            assert!(slot.id == src.id || slot.is_none());

            let space = u8::MAX - slot.count;
            let moved = cmp::min(remaining, space);

            if moved > 0 {
                if slot.is_none() {
                    slot.id = src.id;
                }
                slot.count += moved;
                remaining -= moved;
                updated_slots.push(idx)
            }

            remaining == 0
        };

        if to_slot != NO_SLOT {
            let slot = &mut i.contents_mut()[to_slot as usize];
            if slot.id == src.id || slot.is_none() {
                move_into(slot, to_slot);
            }
        } else {
            for (idx, slot) in i.contents_mut().iter_mut().enumerate()
                    .filter(|&(_, ref s)| s.id == src.id || s.is_none()) {
                let done = move_into(slot, idx as u8);
                if done {
                    break;
                }
            }
        }
    }

    // Update source inventory.
    let total_moved = to_move - remaining;
    {
        let mut wf = DummyFragment::new(&mut eng.world);
        let mut i = unwrap!(wf.get_inventory_mut(from_iid));
        let slot = &mut i.contents_mut()[from_slot as usize];
        slot.count -= total_moved;
        if slot.count == 0 {
            *slot = Item::none();
        }
    }

    // Send messages.
    if total_moved > 0 {
        logic::inventory::on_update(eng.refine(), from_iid, from_slot);
        for &idx in updated_slots.iter() {
            logic::inventory::on_update(eng.refine(), to_iid, idx);
        }
    }

    Ok(total_moved)
}

pub fn bulk_add(eng: &mut Engine,
                iid: InventoryId,
                item_id: ItemId,
                count: u16) -> StrResult<u16> {
    let mut remaining = count;

    // Update destination, keeping track of which slots were updated and how much was moved.
    let mut updated_slots = SmallVec::new();
    {
        let mut wf = DummyFragment::new(&mut eng.world);
        let mut i = unwrap!(wf.get_inventory_mut(iid));
        // Cannot fail past this point.

        for (idx, slot) in i.contents_mut().iter_mut().enumerate()
                .filter(|&(_, ref s)| s.id == item_id || s.is_none()) {
            let space = u8::MAX - slot.count;
            // Final cast never truncates because `space` is already a u8.
            let moved = cmp::min(remaining, space as u16) as u8;
            info!("slot {}: remaining = {}, space = {}, moved = {}",
                  idx, remaining, space, moved);

            if moved > 0 {
                if slot.is_none() {
                    slot.id = item_id;
                }
                slot.count += moved;
                remaining -= moved as u16;
                updated_slots.push(idx as u8)
            }

            if remaining == 0 {
                break;
            }
        }
    }

    let total_moved = count - remaining;
    // Send messages.
    if total_moved > 0 {
        for &idx in updated_slots.iter() {
            logic::inventory::on_update(eng.refine(), iid, idx);
        }
    }

    Ok(total_moved)
}

pub fn bulk_remove(eng: &mut Engine,
                   iid: InventoryId,
                   item_id: ItemId,
                   count: u16) -> StrResult<u16> {
    let mut remaining = count;

    // Update destination, keeping track of which slots were updated and how much was moved.
    let mut updated_slots = SmallVec::new();
    {
        let mut wf = DummyFragment::new(&mut eng.world);
        let mut i = unwrap!(wf.get_inventory_mut(iid));
        // Cannot fail past this point.

        for (idx, slot) in i.contents_mut().iter_mut().enumerate()
                .filter(|&(_, ref s)| s.id == item_id) {
            // Final cast never truncates because `slot.count` is already a u8.
            let moved = cmp::min(remaining, slot.count as u16) as u8;
            info!("slot {}: remaining = {}, count = {}, moved = {}",
                  idx, remaining, slot.count, moved);

            if moved > 0 {
                slot.count -= moved;
                if slot.count == 0 {
                    *slot = Item::none();
                }
                remaining -= moved as u16;
                updated_slots.push(idx as u8)
            }

            if remaining == 0 {
                break;
            }
        }
    }

    let total_moved = count - remaining;
    // Send messages.
    if total_moved > 0 {
        for &idx in updated_slots.iter() {
            logic::inventory::on_update(eng.refine(), iid, idx);
        }
    }

    Ok(total_moved)
}


pub fn craft_recipe(eng: &mut Engine,
                    station_sid: StructureId,
                    iid: InventoryId,
                    recipe_id: RecipeId,
                    count: u16) -> StrResult<()> {
    let recipe = unwrap!(eng.data.recipes.get_recipe(recipe_id));

    let _ = station_sid; // TODO

    let real_count = {
        let mut wf = DummyFragment::new(&mut eng.world);
        let mut i = unwrap!(wf.get_inventory_mut(iid));

        let mut count = count;

        for (&item_id, &num_required) in recipe.inputs.iter() {
            count = cmp::min(count, i.count(item_id) / num_required as u16);
        }

        // TODO: this calculation is wrong for multiple outputs
        // It counts Item::Empty as 255 available space for *each* output
        for (&item_id, &num_produced) in recipe.outputs.iter() {
            count = cmp::min(count, i.count_space(item_id) / num_produced as u16);
        }

        count
    };

    if real_count > 0 {
        for (&item_id, &num_required) in recipe.inputs.iter() {
            bulk_remove(eng, iid, item_id, real_count * num_required as u16);
        }

        for (&item_id, &num_produced) in recipe.outputs.iter() {
            bulk_add(eng, iid, item_id, real_count * num_produced as u16);
        }
    }
    Ok(())
}
