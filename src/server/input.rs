use std::collections::hash_map::{self, HashMap};
use std::mem;
use types::*;

use msg::ExtraArg;

pub use libcommon_movement::{
    InputBits,
    INPUT_LEFT,
    INPUT_RIGHT,
    INPUT_UP,
    INPUT_DOWN,
    INPUT_RUN,
    INPUT_HOLD,
    INPUT_DIR_MASK,
};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Interact,
    UseItem(ItemId),
    UseAbility(ItemId),
}


/// Track the latest input from each client.
pub struct Input {
    pending_input: HashMap<ClientId, (InputBits, u16)>,
    pending_action: HashMap<ClientId, (Action, Option<ExtraArg>)>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            pending_input: HashMap::new(),
            pending_action: HashMap::new(),
        }
    }

    pub fn schedule_input(&mut self, cid: ClientId, input: InputBits) {
        let mut i = self.pending_input.entry(cid).or_insert((input, 0));
        i.0 = input;
        // If this ever wraps, the client will get very confused, but that's their own fault for
        // spamming 65,000 inputs in a single tick.
        i.1 = i.1.wrapping_add(1);
    }

    pub fn schedule_action(&mut self, cid: ClientId, action: Action, args: Option<ExtraArg>) {
        self.pending_action.insert(cid, (action, args));
    }

    pub fn inputs(&mut self) -> hash_map::IntoIter<ClientId, (InputBits, u16)> {
        mem::replace(&mut self.pending_input, HashMap::new()).into_iter()
    }

    pub fn actions(&mut self) -> hash_map::IntoIter<ClientId, (Action, Option<ExtraArg>)> {
        mem::replace(&mut self.pending_action, HashMap::new()).into_iter()
    }
}
