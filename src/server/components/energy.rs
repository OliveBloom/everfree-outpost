use std::collections::HashMap;

use types::*;
use libcommon::Gauge;

use components::{Component, EngineComponents};
use world::Entity;
use world::bundle;
use world::extra::Value;


pub struct Energy {
    map: HashMap<EntityId, Gauge>,
}

impl Component<Entity> for Energy {
    fn get<'a>(eng: &'a EngineComponents) -> &'a Self {
        &eng.energy
    }

    fn get_mut<'a>(eng: &'a mut EngineComponents) -> &'a mut Self {
        &mut eng.energy
    }

    fn export(&self, id: EntityId, b: &mut bundle::Entity, now: Time) {
        // Remove if present, so that deleting the gauge and saving will erase the entry from
        // `extras`.
        b.extra.remove("energy");

        let g = unwrap_or!(self.map.get(&id));

        let mut e = b.extra.set_hash("energy");
        e.borrow().set("cur", Value::Int(g.get(now) as i64));
        e.borrow().set("max", Value::Int(g.max() as i64));
    }

    fn import(&mut self, id: EntityId, b: &bundle::Entity, now: Time) {
        let e = unwrap_or!(b.extra.get("energy").and_then(|v| v.as_hash()));
        let cur = unwrap_or!(e.get("cur").and_then(|v| v.as_value()).and_then(|v| v.as_int()));
        let max = unwrap_or!(e.get("max").and_then(|v| v.as_value()).and_then(|v| v.as_int()));

        let g = Gauge::new(cur as i32, (0, 0), now, 0, max as i32);
        self.map.insert(id, g);
    }

    fn cleanup(&mut self, id: EntityId) {
        // Does the right thing whether or not `id` is present.
        self.map.remove(&id);
    }
}

impl Energy {
    pub fn new() -> Energy {
        Energy {
            map: HashMap::new(),
        }
    }

    pub fn init(&mut self, id: EntityId, max: i32) {
        if self.map.contains_key(&id) {
            return;
        }

        let g = Gauge::new(max, (0, 0), 0, 0, max);
        self.map.insert(id, g);
    }

    pub fn get(&self, id: EntityId, now: Time) -> i32 {
        self.map.get(&id).map_or(0, |g| g.get(now))
    }

    pub fn take(&mut self, id: EntityId, amount: i32, now: Time) -> bool {
        let g = unwrap_or!(self.get_gauge_mut(id), return false);
        if g.get(now) < amount {
            return false;
        }
        g.adjust(-amount, now);
        true
    }

    pub fn get_gauge_mut(&mut self, id: EntityId) -> Option<&mut Gauge> {
        self.map.get_mut(&id)
    }

    pub fn gauge_mut(&mut self, id: EntityId) -> &mut Gauge {
        self.get_gauge_mut(id)
            .unwrap_or_else(|| panic!("no energy gauge for entity {:?}", id))
    }
}
