use std::borrow::ToOwned;
use std::collections::HashSet;
use std::mem::replace;

use types::*;

use input::InputBits;
use world::EntityAttachment;

use world::{World, Client};
use world::extra::Extra;
use world::ops::{self, OpResult};


pub fn create(w: &mut World,
              name: &str) -> OpResult<ClientId> {
    let c = Client {
        name: name.to_owned(),
        pawn: None,
        current_input: InputBits::empty(),

        extra: Extra::new(),
        stable_id: NO_STABLE_ID,
        child_entities: HashSet::new(),
        child_inventories: HashSet::new(),

        version: w.snapshot.version() + 1,
    };

    let cid = unwrap!(w.clients.insert(c));
    Ok(cid)
}

pub fn create_unchecked(w: &mut World) -> ClientId {
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

pub fn destroy(w: &mut World,
               cid: ClientId) -> OpResult<()> {
    let c = unwrap!(w.clients.remove(cid));
    // Further lookup failures indicate an invariant violation.
    w.snapshot.record_client(cid, &c);

    for &eid in c.child_entities.iter() {
        // TODO: do we really want .unwrap() here?
        ops::entity::destroy(w, eid).unwrap();
    }

    for &iid in c.child_inventories.iter() {
        ops::inventory::destroy(w, iid).unwrap();
    }

    Ok(())
}

pub fn set_pawn(w: &mut World,
                cid: ClientId,
                eid: EntityId) -> OpResult<Option<EntityId>> {
    try!(ops::entity::attach(w, eid, EntityAttachment::Client(cid)));
    let old_eid;

    {
        let c = unwrap!(w.clients.get_mut(cid));
        // We know 'eid' is valid because the 'entity_attach' above succeeded.
        old_eid = replace(&mut c.pawn, Some(eid));
    }

    Ok(old_eid)
}

pub fn clear_pawn(w: &mut World,
                  cid: ClientId) -> OpResult<Option<EntityId>> {
    let old_eid;
    {
        let c = unwrap!(w.clients.get_mut(cid));
        // NB: Keep this behavior in sync with entity_destroy.
        old_eid = replace(&mut c.pawn, None);
    }

    Ok(old_eid)
}
