use std::cmp;

use types::*;
use util::StrResult;

use dialogs::DialogType;
use engine::Engine;
use engine::split::EngineRef;
use engine::split2::Coded;
use logic;
use messages::{ClientResponse, Dialog};
use world;
use world::object::*;
use vision;


pub fn open_inventory(mut eng: EngineRef, cid: ClientId, iid: InventoryId) -> StrResult<()> {
    // Check that IDs are valid.
    unwrap!(eng.world().get_client(cid));
    unwrap!(eng.world().get_inventory(iid));

    logic::inventory::subscribe(eng.borrow().unwrap().refine(), cid, iid);
    eng.messages_mut().send_client(cid, ClientResponse::OpenDialog(Dialog::Inventory(iid)));

    Ok(())
}

pub fn open_container(mut eng: EngineRef,
                      cid: ClientId,
                      iid1: InventoryId,
                      iid2: InventoryId) -> StrResult<()> {
    // Check that IDs are valid.
    unwrap!(eng.world().get_client(cid));
    unwrap!(eng.world().get_inventory(iid1));
    unwrap!(eng.world().get_inventory(iid1));

    logic::inventory::subscribe(eng.borrow().unwrap().refine(), cid, iid1);
    logic::inventory::subscribe(eng.borrow().unwrap().refine(), cid, iid2);
    eng.borrow().unwrap().dialogs.set_dialog(cid,
                                             DialogType::Container(StructureId(0), iid1, iid2),
                                             |_,_| ());
    eng.messages_mut().send_client(cid, ClientResponse::OpenDialog(Dialog::Container(iid1, iid2)));

    Ok(())
}

pub fn open_crafting(mut eng: EngineRef,
                     cid: ClientId,
                     sid: StructureId,
                     iid: InventoryId) -> StrResult<()> {
    // Check that IDs are valid.
    unwrap!(eng.world().get_client(cid));
    unwrap!(eng.world().get_inventory(iid));

    let template_id = {
        let s = unwrap!(eng.world().get_structure(sid));
        s.template_id()
    };

    logic::inventory::subscribe(eng.borrow().unwrap().refine(), cid, iid);
    eng.borrow().unwrap().dialogs.set_dialog(cid,
                                             DialogType::Crafting(sid, iid),
                                             |_,_| ());
    let dialog = Dialog::Crafting(template_id, sid, iid);
    eng.messages_mut().send_client(cid, ClientResponse::OpenDialog(dialog));

    Ok(())
}

pub fn set_main_inventories(mut eng: EngineRef,
                            cid: ClientId,
                            item_iid: InventoryId,
                            ability_iid: InventoryId) -> StrResult<()> {
    // Check that IDs are valid.
    unwrap!(eng.world().get_client(cid));
    unwrap!(eng.world().get_inventory(item_iid));
    unwrap!(eng.world().get_inventory(ability_iid));

    logic::inventory::subscribe(eng.borrow().unwrap().refine(), cid, item_iid);
    logic::inventory::subscribe(eng.borrow().unwrap().refine(), cid, ability_iid);
    eng.messages_mut().send_client(cid, ClientResponse::MainInventory(item_iid));
    eng.messages_mut().send_client(cid, ClientResponse::AbilityInventory(ability_iid));

    Ok(())
}


pub fn move_items(eng: &mut Engine,
                  from_iid: InventoryId,
                  to_iid: InventoryId,
                  item_id: ItemId,
                  count: u16) -> StrResult<u16> {
    let avail = unwrap!(eng.world.get_inventory(from_iid)).count(item_id);
    let space = unwrap!(eng.world.get_inventory(to_iid)).count_space(item_id);
    let actual = cmp::min(cmp::min(avail, space), count);

    // OK: inventory IDs have already been checked.
    world::Fragment::inventory_mut(&mut eng.as_ref().as_world_fragment(), from_iid)
         .bulk_remove(item_id, actual);
    world::Fragment::inventory_mut(&mut eng.as_ref().as_world_fragment(), to_iid)
         .bulk_add(item_id, actual);

    Ok(actual)
}

pub fn move_items2(eng: &mut Engine,
                   from_iid: InventoryId,
                   from_slot: u8,
                   to_iid: InventoryId,
                   to_slot: u8,
                   count: u8) -> StrResult<u8> {
    let mut eng_ref = eng.as_ref();
    let mut wf = eng_ref.as_world_fragment();

    info!("move {} from {:?}.{} to {:?}.{}", count, from_iid, from_slot, to_iid, to_slot);
    let proposed = {
        let i = unwrap!(wf.world().get_inventory(from_iid));
        try!(i.transfer_propose(from_slot, count))
    };
    info!("  proposal: {:?}", proposed);

    let actual = {
        let mut i = unwrap!(world::Fragment::get_inventory_mut(&mut wf, to_iid));
        try!(i.transfer_receive(to_slot, proposed))
    };
    info!("  actual: {:?}", actual);

    {
        // OK: already checked to_iid
        let mut i = world::Fragment::inventory_mut(&mut wf, from_iid);
        // Should never fail, but it's good to check.
        warn_on_err!(i.transfer_commit(from_slot, actual));
    }
    info!("  commited transfer");

    Ok(actual.count())
}


pub fn craft_recipe(eng: &mut Engine,
                    station_sid: StructureId,
                    iid: InventoryId,
                    recipe_id: RecipeId,
                    count: u16) -> StrResult<()> {
    let recipe = unwrap!(eng.data.recipes.get_recipe(recipe_id));

    let _ = station_sid; // TODO
    let mut eng_ref = eng.as_ref();
    let mut wf = eng_ref.as_world_fragment();
    let mut i = unwrap!(world::Fragment::get_inventory_mut(&mut wf, iid));

    let real_count = {
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
            i.bulk_remove(item_id, real_count * num_required as u16);
        }

        for (&item_id, &num_produced) in recipe.outputs.iter() {
            i.bulk_add(item_id, real_count * num_produced as u16);
        }
    }
    Ok(())
}
