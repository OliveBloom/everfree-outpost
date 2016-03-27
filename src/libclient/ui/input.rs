pub enum KeyAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    Select,
    Cancel,

    SetHotbar(i8),
}

impl KeyAction {
    pub fn from_code(code: u8) -> Option<KeyAction> {
        use self::KeyAction::*;
        match code {
            0 => Some(MoveLeft),
            1 => Some(MoveRight),
            2 => Some(MoveUp),
            3 => Some(MoveDown),

            4 => Some(Select),
            5 => Some(Cancel),

            11 ... 19 => Some(SetHotbar(code as i8 - 11)),

            _ => None,
        }
    }
}
