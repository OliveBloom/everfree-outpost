use std::prelude::v1::*;
use common_proto::game::Request;

use Time;

#[macro_use] pub mod gl;


pub trait Platform {
    type GL: gl::Context;
    fn gl(&mut self) -> &mut Self::GL;

    type Config: Config;
    fn config(&self) -> &Self::Config;
    fn config_mut(&mut self) -> &mut Self::Config;

    fn set_cursor(&mut self, cursor: Cursor);

    fn send_message(&mut self, msg: Request);

    fn get_time(&self) -> Time;
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cursor {
    /// Default cursor
    Normal = 0,
    /// Used when dragging something
    Drag = 1,
    /// Used when dragging something over an invalid drop location
    DragInvalid = 2,
}


pub trait PlatformObj {
    // GlContext is not object-safe

    fn config(&self) -> &Config;
    fn config_mut(&mut self) -> &mut Config;

    fn set_cursor(&mut self, cursor: Cursor);

    fn send_message(&mut self, msg: Request);

    fn get_time(&self) -> Time;
}

impl<P: Platform> PlatformObj for P {
    fn config(&self) -> &Config {
        Platform::config(self)
    }

    fn config_mut(&mut self) -> &mut Config {
        Platform::config_mut(self)
    }


    fn set_cursor(&mut self, cursor: Cursor) {
        Platform::set_cursor(self, cursor);
    }

    fn send_message(&mut self, msg: Request) {
        Platform::send_message(self, msg);
    }

    fn get_time(&self) -> Time {
        Platform::get_time(self)
    }
}


pub enum ConfigKey {
    DebugShowPanel,
    HotbarItemName(u8),
    HotbarIsItem(u8),
    HotbarActiveItem,
    HotbarActiveAbility,
    RenderNames,
    ScaleUI,
    ScaleWorld,
}

impl ConfigKey {
    pub fn to_string(&self) -> String {
        use self::ConfigKey::*;
        match *self {
            DebugShowPanel => "debug_show_panel".into(),
            HotbarItemName(idx) => format!("hotbar.names.{}", idx),
            HotbarIsItem(idx) => format!("hotbar.is_item.{}", idx),
            HotbarActiveItem => "hotbar.active_item".into(),
            HotbarActiveAbility => "hotbar.active_ability".into(),
            RenderNames => "render_names".into(),
            ScaleUI => "scale_ui".into(),
            ScaleWorld => "scale_world".into(),
        }
    }
}

pub trait Config {
    fn get_int(&self, key: ConfigKey) -> i32;
    fn set_int(&mut self, key: ConfigKey, value: i32);

    fn get_str(&self, key: ConfigKey) -> String;
    fn set_str(&mut self, key: ConfigKey, value: &str);

    fn clear(&mut self, key: ConfigKey);
}

