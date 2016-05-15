use std::collections::hash_map::{self, HashMap};
use std::mem;
use types::*;

use msg::ExtraArg;


bitflags! {
    pub flags InputBits: u16 {
        const INPUT_LEFT =      0x0001,
        const INPUT_RIGHT =     0x0002,
        const INPUT_UP =        0x0004,
        const INPUT_DOWN =      0x0008,
        const INPUT_RUN =       0x0010,
    }
}

impl InputBits {
    pub fn to_velocity(&self) -> V3 {
        let x =
            if self.contains(INPUT_LEFT) { -1 } else { 0 } +
            if self.contains(INPUT_RIGHT) { 1 } else { 0 };
        let y =
            if self.contains(INPUT_UP) { -1 } else { 0 } +
            if self.contains(INPUT_DOWN) { 1 } else { 0 };
        // TODO: player speed handling shouldn't be here
        let speed = if self.contains(INPUT_RUN) { 150 } else { 50 };
        V3::new(x, y, 0) * scalar(speed)
    }
}


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
