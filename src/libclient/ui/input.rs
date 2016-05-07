use std::prelude::v1::*;
use std::boxed::FnBox;

use client::ClientObj;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    Select,
    Cancel,

    SetHotbar(i8),

    ToggleDebugPanel,
}

bitflags! {
    pub flags Modifiers: u8 {
        const ModShift =    0x01,
    }
}

impl KeyAction {
    pub fn from_code(code: u8) -> Option<KeyAction> {
        use self::KeyAction::*;
        match code {
            0 => Some(MoveLeft),
            1 => Some(MoveRight),
            2 => Some(MoveUp),
            3 => Some(MoveDown),

            20 => Some(Select),
            21 => Some(Cancel),

            31 ... 39 => Some(SetHotbar(code as i8 - 31)),

            114 => Some(ToggleDebugPanel),

            _ => None,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyEvent {
    pub code: KeyAction,
    pub mods: Modifiers,
}

impl KeyEvent {
    pub fn new(code: KeyAction, mods: Modifiers) -> KeyEvent {
        KeyEvent {
            code: code,
            mods: mods,
        }
    }

    pub fn shift(&self) -> bool {
        self.mods.contains(ModShift)
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
