use std::prelude::v1::*;
use std::mem;
use types::*;
use common::Gauge;
use common_movement::InputBits;

use entity::{Entity, Entities, Motion};

pub enum Activity {
    Walk,
    // Fly,
    Emote,
    Busy,
}

pub struct PawnInfo {
    id: Option<EntityId>,

    // Shadow state for the real pawn Entity.  The actual values sent from the server are saved
    // here, so we can overwrite the data in Entities but still restore it if the pawn changes.
    name: Option<String>, 
    real_motion: Motion,
    effective_motion: Motion,

    cur_input: InputBits,
    energy: Gauge,
    activity: Activity,
}

impl PawnInfo {
    pub fn new() -> PawnInfo {
        PawnInfo {
            id: None,

            name: None,
            real_motion: Motion::new(),
            effective_motion: Motion::new(),

            cur_input: InputBits::empty(),
            energy: Gauge::new(0, (0, 1), 0, 0, 1),
            activity: Activity::Walk,
        }
    }

    pub fn id(&self) -> Option<EntityId> {
        self.id
    }

    pub fn is(&self, id: EntityId) -> bool {
        Some(id) == self.id
    }

    pub fn get<'a>(&self, entities: &'a Entities) -> Option<&'a Entity> {
        self.id.and_then(|eid| entities.get(eid))
    }


    pub fn set_id(&mut self, id: EntityId, entities: &mut Entities) {
        if self.id.is_some() {
            self.clear_id(entities);
        }
        self.id = Some(id);

        if let Some(e) = entities.get_mut(id) {
            self.acquire(e);
        }
    }

    pub fn clear_id(&mut self, entities: &mut Entities) {
        if self.id.is_none() {
            return;
        }

        let id = self.id.unwrap();
        self.id = None;

        if let Some(e) = entities.get_mut(id) {
            self.release(e);
        }
    }

    pub fn on_create(&mut self, entities: &mut Entities) {
        if let Some(id) = self.id {
            let e = entities.get_mut(id)
                .expect("entity should exist before calling on_create");
            self.acquire(e);
        }
    }


    fn acquire(&mut self, e: &mut Entity) {
        self.name = mem::replace(&mut e.name, None);
        self.real_motion = e.motion.clone();
        self.effective_motion = e.motion.clone();
    }

    fn release(&mut self, e: &mut Entity) {
        e.name = mem::replace(&mut self.name, None);
        e.motion = self.real_motion.clone();
    }
}

