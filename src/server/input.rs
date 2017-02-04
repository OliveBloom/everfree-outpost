use std::cmp;
use std::collections::hash_map::{self, HashMap};
use std::mem;
use std::ptr;

use libcommon_proto::ExtraArg;

use types::*;
use util;


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


const QUEUE_SIZE: usize = 8;

/// Track the latest input from each client.
pub struct Input {
    pending_input_maps: [HashMap<ClientId, (InputBits, u16)>; QUEUE_SIZE],
    input_index: i32,
    pending_action: HashMap<ClientId, (Action, Option<ExtraArg>)>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            pending_input_maps: unsafe { util::fixed_array_with(HashMap::new) },
            input_index: 0,
            pending_action: HashMap::new(),
        }
    }

    pub fn schedule_input(&mut self, cid: ClientId, offset: i32, input: InputBits) {
        let offset = cmp::max(0, cmp::min(offset, QUEUE_SIZE as i32 - 1));
        let slot = (self.input_index + offset) as usize % QUEUE_SIZE;
        let mut i = self.pending_input_maps[slot].entry(cid).or_insert((input, 0));
        i.0 = input;
        // If this ever wraps, the client will get very confused, but that's their own fault for
        // spamming 65,000 inputs in a single tick.
        i.1 = i.1.wrapping_add(1);
    }

    pub fn schedule_action(&mut self, cid: ClientId, action: Action, args: Option<ExtraArg>) {
        self.pending_action.insert(cid, (action, args));
    }

    pub fn inputs(&mut self) -> hash_map::IntoIter<ClientId, (InputBits, u16)> {
        let map = &mut self.pending_input_maps[self.input_index as usize];
        let iter = mem::replace(map, HashMap::new()).into_iter();
        self.input_index = (self.input_index + 1) % QUEUE_SIZE as i32;
        iter
    }

    pub fn actions(&mut self) -> hash_map::IntoIter<ClientId, (Action, Option<ExtraArg>)> {
        mem::replace(&mut self.pending_action, HashMap::new()).into_iter()
    }
}
