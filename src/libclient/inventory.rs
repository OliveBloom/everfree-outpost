use std::prelude::v1::*;
use std::collections::btree_map::BTreeMap;
use std::ops::Index;


pub type InventoryId = u32;

pub struct Item {
    pub id: u16,
    pub quantity: u8,
}

impl Item {
    pub fn new(id: u16, quantity: u8) -> Item {
        Item {
            id: id,
            quantity: quantity,
        }
    }
}

pub struct Inventory {
    pub items: Box<[Item]>,
}

impl Inventory {
    fn new(items: Box<[Item]>) -> Inventory {
        Inventory { items: items }
    }
}

pub struct Inventories {
    map: BTreeMap<InventoryId, Inventory>,
    main_inv_id: Option<InventoryId>,
    ability_inv_id: Option<InventoryId>,
}

impl Inventories {
    pub fn new() -> Inventories {
        Inventories {
            map: BTreeMap::new(),
            main_inv_id: None,
            ability_inv_id: None,
        }
    }

    pub fn insert(&mut self, id: InventoryId, items: Box<[Item]>) {
        self.map.insert(id, Inventory::new(items));
    }

    pub fn remove(&mut self, id: InventoryId) -> Inventory {
        self.map.remove(&id).unwrap()
    }

    pub fn update(&mut self,
                  id: InventoryId,
                  slot: usize,
                  item: Item) {
        if let Some(inv) = self.map.get_mut(&id) {
            inv.items[slot] = item;
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.main_inv_id = None;
        self.ability_inv_id = None;
    }


    pub fn main_inventory(&self) -> Option<&Inventory> {
        self.main_inv_id.map(|x| &self.map[&x])
    }

    pub fn main_id(&self) -> Option<InventoryId> {
        self.main_inv_id
    }

    pub fn set_main_id(&mut self, id: InventoryId) {
        self.main_inv_id = Some(id);
    }

    pub fn clear_main_id(&mut self) {
        self.main_inv_id = None;
    }

    pub fn ability_inventory(&self) -> Option<&Inventory> {
        self.ability_inv_id.map(|x| &self.map[&x])
    }

    pub fn ability_id(&self) -> Option<InventoryId> {
        self.ability_inv_id
    }

    pub fn set_ability_id(&mut self, id: InventoryId) {
        self.ability_inv_id = Some(id);
    }

    pub fn clear_ability_id(&mut self) {
        self.ability_inv_id = None;
    }
}

impl Index<InventoryId> for Inventories {
    type Output = Inventory;
    fn index(&self, idx: InventoryId) -> &Inventory {
        &self.map[&idx]
    }
}
