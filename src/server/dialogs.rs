use std::collections::HashMap;

use types::*;

use pubsub::{self, PubSub};


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TargetId {
    Inventory(InventoryId),
    Structure(StructureId),
}

impl pubsub::Name for TargetId {
    fn min_bound() -> TargetId { TargetId::Inventory(pubsub::Name::min_bound()) }
    fn max_bound() -> TargetId { TargetId::Structure(pubsub::Name::max_bound()) }
}

pub struct Dialogs {
    map: HashMap<ClientId, DialogType>,
    ps: PubSub<TargetId, (), ClientId>,
}

impl Dialogs {
    pub fn new() -> Dialogs {
        Dialogs {
            map: HashMap::new(),
            ps: PubSub::new(),
        }
    }

    pub fn clear_dialog<F>(&mut self, cid: ClientId, mut f: F)
            where F: FnMut(TargetId) {
        let dialog = unwrap_or!(self.map.remove(&cid));
        dialog.iter_targets(|t| {
            self.ps.unsubscribe_publisher(cid, t, |&tt,_| f(tt));
        });
        info!("cleared dialog for {:?}", cid);
    }

    pub fn set_dialog<F>(&mut self, cid: ClientId, dialog: DialogType, mut f: F)
            where F: FnMut(TargetId, bool) {
        self.clear_dialog(cid, |t| f(t, false));
        dialog.iter_targets(|t| {
            self.ps.subscribe_publisher(cid, t, |&tt,_| f(tt, true));
        });
        info!("set dialog for {:?} to {:?}", cid, dialog);
        self.map.insert(cid, dialog);
    }

    pub fn clear_inventory_users<F>(&mut self, iid: InventoryId, mut f: F)
            where F: FnMut(ClientId, Option<TargetId>) {
        self.clear_users(TargetId::Inventory(iid), f);
    }

    pub fn clear_structure_users<F>(&mut self, sid: StructureId, mut f: F)
            where F: FnMut(ClientId, Option<TargetId>) {
        self.clear_users(TargetId::Structure(sid), f);
    }

    pub fn clear_users<F>(&mut self, t: TargetId, mut f: F)
            where F: FnMut(ClientId, Option<TargetId>) {
        let mut cids = Vec::new();
        self.ps.message(&t, |_,&cid| {
            cids.push(cid);
        });
        for cid in cids {
            f(cid, None);
            self.clear_dialog(cid, |tt| f(cid, Some(tt)));
        }
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DialogType {
    Inventory(InventoryId),
    Abilities(InventoryId),
    Container(InventoryId, InventoryId),
    Crafting(StructureId, InventoryId),
}

impl DialogType {
    fn iter_targets<F: FnMut(TargetId)>(&self, mut f: F) {
        use self::DialogType::*;
        match *self {
            Inventory(iid) => {
                f(TargetId::Inventory(iid));
            },
            Abilities(iid) => {
                f(TargetId::Inventory(iid));
            },
            Container(iid1, iid2) => {
                f(TargetId::Inventory(iid1));
                f(TargetId::Inventory(iid2));
            },
            Crafting(sid, iid) => {
                f(TargetId::Structure(sid));
                f(TargetId::Inventory(iid));
            },
        }
    }
}

