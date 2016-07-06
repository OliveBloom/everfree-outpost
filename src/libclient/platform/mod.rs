use std::prelude::v1::*;

use inventory::InventoryId;

#[macro_use] pub mod gl;


pub trait Platform {
    type GL: gl::Context;
    fn gl(&mut self) -> &mut Self::GL;

    type Config: Config;
    fn config(&self) -> &Self::Config;
    fn config_mut(&mut self) -> &mut Self::Config;

    fn set_cursor(&mut self, cursor: Cursor);

    fn send_move_item(&mut self,
                      src_inv: InventoryId,
                      src_slot: usize,
                      dest_inv: InventoryId,
                      dest_slot: usize,
                      amount: u8);
    fn send_close_dialog(&mut self);
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

    fn send_move_item(&mut self,
                      src_inv: InventoryId,
                      src_slot: usize,
                      dest_inv: InventoryId,
                      dest_slot: usize,
                      amount: u8);
    fn send_close_dialog(&mut self);
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

    fn send_move_item(&mut self,
                      src_inv: InventoryId,
                      src_slot: usize,
                      dest_inv: InventoryId,
                      dest_slot: usize,
                      amount: u8) {
        Platform::send_move_item(self, src_inv, src_slot, dest_inv, dest_slot, amount);
    }

    fn send_close_dialog(&mut self) {
        Platform::send_close_dialog(self);
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

