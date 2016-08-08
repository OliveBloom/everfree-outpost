use std::prelude::v1::*;

use input::{Key, Modifiers, KeyEvent};
use input::MOD_SHIFT;


// UI actions
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

impl KeyAction {
    pub fn from_key(key: Key) -> Option<KeyAction> {
        use self::KeyAction::*;
        match key {
            Key::MoveLeft => Some(MoveLeft),
            Key::MoveRight => Some(MoveRight),
            Key::MoveUp => Some(MoveUp),
            Key::MoveDown => Some(MoveDown),

            Key::Interact => Some(Select),
            Key::Select => Some(Select),
            Key::Cancel => Some(Cancel),

            Key::Hotbar(idx) => Some(SetHotbar(idx)),

            Key::ToggleDebugPanel => Some(ToggleDebugPanel),

            _ => None,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ActionEvent {
    pub code: KeyAction,
    pub mods: Modifiers,
}

impl ActionEvent {
    pub fn new(code: KeyAction, mods: Modifiers) -> ActionEvent {
        ActionEvent {
            code: code,
            mods: mods,
        }
    }

    pub fn from_key_event(evt: KeyEvent) -> Option<ActionEvent> {
        KeyAction::from_key(evt.code)
            .map(|action| ActionEvent::new(action, evt.mods))
    }

    pub fn shift(&self) -> bool {
        self.mods.contains(MOD_SHIFT)
    }
}
