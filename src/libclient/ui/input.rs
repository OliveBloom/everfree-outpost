use std::prelude::v1::*;
use std::boxed::FnBox;

use client::ClientObj;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyAction {
    MoveLeft(u8),
    MoveRight(u8),
    MoveUp(u8),
    MoveDown(u8),

    Select,
    Cancel,

    SetHotbar(i8),

    ToggleDebugPanel,
}

impl KeyAction {
    pub fn from_code(code: u8) -> Option<KeyAction> {
        use self::KeyAction::*;
        match code {
            0 => Some(MoveLeft(1)),
            1 => Some(MoveRight(1)),
            2 => Some(MoveUp(1)),
            3 => Some(MoveDown(1)),

            10 => Some(MoveLeft(10)),
            11 => Some(MoveRight(10)),
            12 => Some(MoveUp(10)),
            13 => Some(MoveDown(10)),

            20 => Some(Select),
            21 => Some(Cancel),

            31 ... 39 => Some(SetHotbar(code as i8 - 31)),

            114 => Some(ToggleDebugPanel),

            _ => None,
        }
    }
}


pub enum EventStatus {
    Unhandled,
    Handled,
    //UseDefault,
    // TODO: would like to use `&mut Client`, but don't want to thread <GL> all around
    Action(Box<FnBox(&mut ClientObj)>),
}

impl EventStatus {
    pub fn action<F: FnOnce(&mut ClientObj)+'static>(f: F) -> EventStatus {
        EventStatus::Action(box f)
    }

    pub fn is_handled(&self) -> bool {
        match *self {
            EventStatus::Unhandled => false,
            _ => true,
        }
    }
}
