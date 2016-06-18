use std::borrow::ToOwned;
use std::collections::HashSet;
use std::mem::replace;

use types::*;

use input::InputBits;
use world::EntityAttachment;

use world::Client;
use world::{Fragment, Hooks};
use world::extra::Extra;
use world::ops::{self, OpResult};


pub fn create<'d, F>(f: &mut F,
                     name: &str) -> OpResult<ClientId>
        where F: Fragment<'d> {
    let c = Client {
        name: name.to_owned(),
        pawn: None,
        current_input: InputBits::empty(),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        child_entities: HashSet::new(),
        child_inventories: HashSet::new(),

        version: f.world().snapshot.version() + 1,
    };

    let cid = unwrap!(f.world_mut().clients.insert(c));
    Ok(cid)
}

pub fn create_unchecked<'d, F>(f: &mut F) -> ClientId
        where F: Fragment<'d> {
    let w = f.world_mut();
    let cid = w.clients.insert(Client {
        name: String::new(),
        pawn: None,
        current_input: InputBits::empty(),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        child_entities: HashSet::new(),
        child_inventories: HashSet::new(),

        version: w.snapshot.version() + 1,
    }).unwrap();     // Shouldn't fail when stable_id == NO_STABLE_ID
    cid
}

pub fn destroy<'d, F>(f: &mut F,
                      cid: ClientId) -> OpResult<()>
        where F: Fragment<'d> {
    let c = unwrap!(f.world_mut().clients.remove(cid));
    // Further lookup failures indicate an invariant violation.
    f.world_mut().snapshot.record_client(cid, &c);

    for &eid in c.child_entities.iter() {
        // TODO: do we really want .unwrap() here?
        ops::entity::destroy(f, eid).unwrap();
    }

    for &iid in c.child_inventories.iter() {
        ops::inventory::destroy(f, iid).unwrap();
    }

    Ok(())
}

pub fn set_pawn<'d, F>(f: &mut F,
                       cid: ClientId,
                       eid: EntityId) -> OpResult<Option<EntityId>>
        where F: Fragment<'d> {
    try!(ops::entity::attach(f, eid, EntityAttachment::Client(cid)));
    let old_eid;

    {
        let c = unwrap!(f.world_mut().clients.get_mut(cid));
        // We know 'eid' is valid because the 'entity_attach' above succeeded.
        old_eid = replace(&mut c.pawn, Some(eid));
    }

    Ok(old_eid)
}

pub fn clear_pawn<'d, F>(f: &mut F,
                         cid: ClientId) -> OpResult<Option<EntityId>>
        where F: Fragment<'d> {
    let old_eid;
    {
        let c = unwrap!(f.world_mut().clients.get_mut(cid));
        // NB: Keep this behavior in sync with entity_destroy.
        old_eid = replace(&mut c.pawn, None);
    }

    Ok(old_eid)
}
