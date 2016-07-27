use std::mem::replace;

use types::*;
use util;

use world::{World, Inventory, InventoryAttachment, InventoryFlags, Item};
use world::extra::Extra;
use world::ops::OpResult;


// Inventory size (number of slots) is capped at 255
pub fn create(w: &mut World, size: u8) -> OpResult<InventoryId> {
    let i = Inventory {
        contents: util::make_array(Item::none(), size as usize),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: InventoryFlags::empty(),
        attachment: InventoryAttachment::World,

        version: w.snapshot.version() + 1,
    };

    let iid = unwrap!(w.inventories.insert(i));
    Ok(iid)
}

pub fn create_unchecked(w: &mut World) -> InventoryId {
    let iid = w.inventories.insert(Inventory {
        contents: util::make_array(Item::none(), 0),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        flags: InventoryFlags::empty(),
        attachment: InventoryAttachment::World,

        version: w.snapshot.version() + 1,
    }).unwrap();     // Shouldn't fail when stable_id == NO_STABLE_ID
    iid
}

pub fn destroy(w: &mut World,
               iid: InventoryId) -> OpResult<()> {
    use world::InventoryAttachment::*;
    let i = unwrap!(w.inventories.remove(iid));
    w.snapshot.record_inventory(iid, &i);

    match i.attachment {
        World => {},
        Client(cid) => {
            if let Some(c) = w.clients.get_mut(cid) {
                c.child_inventories.remove(&iid);
            }
        },
        Entity(eid) => {
            if let Some(e) = w.entities.get_mut(eid) {
                e.child_inventories.remove(&iid);
            }
        },
        Structure(sid) => {
            if let Some(s) = w.structures.get_mut(sid) {
                s.child_inventories.remove(&iid);
            }
        },
    }

    Ok(())
}

pub fn attach(w: &mut World,
              iid: InventoryId,
              new_attach: InventoryAttachment) -> OpResult<InventoryAttachment> {
    use world::InventoryAttachment::*;

    let i = unwrap!(w.inventories.get_mut(iid));

    if new_attach == i.attachment {
        return Ok(new_attach);
    }

    match new_attach {
        World => {},
        Client(cid) => {
            let c = unwrap!(w.clients.get_mut(cid),
                            "can't attach inventory to nonexistent client");
            c.child_inventories.insert(iid);
        },
        Entity(eid) => {
            let e = unwrap!(w.entities.get_mut(eid),
                            "can't attach inventory to nonexistent entity");
            e.child_inventories.insert(iid);
        },
        Structure(sid) => {
            let s = unwrap!(w.structures.get_mut(sid),
                            "can't attach inventory to nonexistent structure");
            s.child_inventories.insert(iid);
        },
    }

    let old_attach = replace(&mut i.attachment, new_attach);

    match old_attach {
        World => {},
        Client(cid) => {
            w.clients[cid].child_inventories.remove(&iid);
        },
        Entity(eid) => {
            w.entities[eid].child_inventories.remove(&iid);
        },
        Structure(sid) => {
            w.structures[sid].child_inventories.remove(&iid);
        },
    }

    Ok(old_attach)
}
