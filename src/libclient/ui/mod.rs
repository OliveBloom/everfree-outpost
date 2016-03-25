use std::prelude::v1::*;

use physics::v3::V2;


mod context;
pub use client_ui_atlas as atlas;
mod state;

mod item;
mod hotbar;

pub use self::context::Context;
pub use self::context::Vertex;
pub use self::state::State;


#[derive(Clone, Copy)]
pub struct WidgetBase;

impl WidgetBase {
    pub fn new() -> WidgetBase {
        WidgetBase
    }
}


pub struct UI {
    context: Context,
    state: State,
    root: hotbar::Hotbar,
    fresh: bool,
}

impl UI {
    pub fn new() -> UI {
        UI {
            context: Context::new(),
            state: State::new(),
            root: hotbar::Hotbar::new(),
            fresh: false,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        self.fresh = false;
        &mut self.state
    }

    pub fn needs_update(&self) -> bool {
        !self.fresh
    }

    pub fn generate_geom(&mut self) -> Vec<Vertex> {
        self.root.render(&mut self.context, &self.state.hotbar, V2::new(0, 0));
        self.fresh = true;
        self.context.take_geometry()
    }
}
