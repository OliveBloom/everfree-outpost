use std::cmp;
use std::mem::replace;
use std::u8;

use types::*;
use util;
use util::SmallVec;

use world::{Inventory, InventoryAttachment, Item};
use world::{Fragment, Hooks, World};
use world::extra::Extra;
use world::ops::OpResult;


// Inventory size (number of slots) is capped at 255
pub fn create<'d, F>(f: &mut F, size: u8) -> OpResult<InventoryId>
        where F: Fragment<'d> {
    let i = Inventory {
        contents: util::make_array(Item::none(), size as usize),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        attachment: InventoryAttachment::World,

        version: f.world().snapshot.version() + 1,
    };

    let iid = unwrap!(f.world_mut().inventories.insert(i));
    Ok(iid)
}

pub fn create_unchecked<'d, F>(f: &mut F) -> InventoryId
        where F: Fragment<'d> {
    let w = f.world_mut();
    let iid = w.inventories.insert(Inventory {
        contents: util::make_array(Item::none(), 0),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        attachment: InventoryAttachment::World,

        version: w.snapshot.version() + 1,
    }).unwrap();     // Shouldn't fail when stable_id == NO_STABLE_ID
    iid
}

pub fn destroy<'d, F>(f: &mut F,
                      iid: InventoryId) -> OpResult<()>
        where F: Fragment<'d> {
    use world::InventoryAttachment::*;
    let i = unwrap!(f.world_mut().inventories.remove(iid));
    f.world_mut().snapshot.record_inventory(iid, &i);

    match i.attachment {
        World => {},
        Client(cid) => {
            if let Some(c) = f.world_mut().clients.get_mut(cid) {
                c.child_inventories.remove(&iid);
            }
        },
        Entity(eid) => {
            if let Some(e) = f.world_mut().entities.get_mut(eid) {
                e.child_inventories.remove(&iid);
            }
        },
        Structure(sid) => {
            if let Some(s) = f.world_mut().structures.get_mut(sid) {
                s.child_inventories.remove(&iid);
            }
        },
    }

    Ok(())
}

pub fn attach<'d, F>(f: &mut F,
                     iid: InventoryId,
                     new_attach: InventoryAttachment) -> OpResult<InventoryAttachment>
        where F: Fragment<'d> {
    use world::InventoryAttachment::*;

    let w = f.world_mut();
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
