use context::Context;

pub enum KeyEvent<Ctx: Context> {
    Down(Ctx::Key),
    Up(Ctx::Key),
}

pub enum MouseEvent<Ctx: Context> {
    Down(Ctx::Button),
    Up(Ctx::Button),
    Move,
}
