use std::prelude::v1::*;

#[macro_use] pub mod gl;


pub trait Platform {
    type GL: gl::Context;
    fn gl(&mut self) -> &mut Self::GL;

    type Config: Config;
    fn config(&self) -> &Self::Config;
    fn config_mut(&mut self) -> &mut Self::Config;
}


pub trait PlatformObj {
    // GlContext is not object-safe

    fn config(&self) -> &Config;
    fn config_mut(&mut self) -> &mut Config;
}

impl<P: Platform> PlatformObj for P {
    fn config(&self) -> &Config {
        Platform::config(self)
    }

    fn config_mut(&mut self) -> &mut Config {
        Platform::config_mut(self)
    }
}


pub enum ConfigKey {
    HotbarItemName(u8),
    HotbarIsItem(u8),
    HotbarActiveItem,
    HotbarActiveAbility,
}

impl ConfigKey {
    pub fn to_string(&self) -> String {
        use self::ConfigKey::*;
        match *self {
            HotbarItemName(idx) => format!("hotbar.names.{}", idx),
            HotbarIsItem(idx) => format!("hotbar.is_item.{}", idx),
            HotbarActiveItem => "hotbar.active_item".into(),
            HotbarActiveAbility => "hotbar.active_ability".into(),
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

