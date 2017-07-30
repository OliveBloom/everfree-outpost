use context::Context;

pub enum KeyEvent<Ctx: Context> {
    Down(Ctx::Key),
    Up(Ctx::Key),
}

impl<Ctx: Context> Clone for KeyEvent<Ctx> {
    fn clone(&self) -> KeyEvent<Ctx> {
        match *self {
            KeyEvent::Down(ref key) => KeyEvent::Down(key.clone()),
            KeyEvent::Up(ref key) => KeyEvent::Up(key.clone()),
        }
    }
}

pub enum KeyInterp {
    /// Cycle focus forward/backward.
    FocusCycle(i8),
    /// Change focus to the next widget in the X direction.
    FocusX(i8),
    /// Change focus to the next widget in the Y direction.
    FocusY(i8),

    /// Activate the focused widget.
    Activate,
}


pub enum MouseEvent<Ctx: Context> {
    Down(Ctx::Button),
    Up(Ctx::Button),
    Move,
    Wheel(i8),
}

impl<Ctx: Context> Clone for MouseEvent<Ctx> {
    fn clone(&self) -> MouseEvent<Ctx> {
        match *self {
            MouseEvent::Down(ref btn) => MouseEvent::Down(btn.clone()),
            MouseEvent::Up(ref btn) => MouseEvent::Up(btn.clone()),
            MouseEvent::Move => MouseEvent::Move,
            MouseEvent::Wheel(dir) => MouseEvent::Wheel(dir),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UIResult<T> {
    Event(T),
    NoEvent,
    Unhandled,
}

impl<T> UIResult<T> {
    pub fn is_handled(&self) -> bool {
        match *self {
            UIResult::Unhandled => false,
            _ => true,
        }
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> UIResult<U> {
        match self {
            UIResult::Event(t) => UIResult::Event(f(t)),
            UIResult::NoEvent => UIResult::NoEvent,
            UIResult::Unhandled => UIResult::Unhandled,
        }
    }

    pub fn and_then<F: FnOnce(T) -> UIResult<U>, U>(self, f: F) -> UIResult<U> {
        match self {
            UIResult::Event(t) => f(t),
            UIResult::NoEvent => UIResult::NoEvent,
            UIResult::Unhandled => UIResult::Unhandled,
        }
    }
}
