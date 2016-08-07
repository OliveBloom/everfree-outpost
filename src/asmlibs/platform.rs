use std::prelude::v1::*;
use common_proto::game::Request;
use common_proto::wire::{WriteTo, Size};

use client::platform;
use client::platform::{ConfigKey, Cursor};

use gl::GL;


mod ffi {
    extern "C" {
        pub fn ap_config_get(key_ptr: *const u8,
                             key_len: usize,
                             value_len_p: *mut usize) -> *mut u8;
        pub fn ap_config_get_int(key_ptr: *const u8,
                                 key_len: usize) -> i32;
        pub fn ap_config_set(key_ptr: *const u8,
                             key_len: usize,
                             value_ptr: *const u8,
                             value_len: usize);
        pub fn ap_config_clear(key_ptr: *const u8,
                               key_len: usize);

        pub fn ap_set_cursor(cursor: u8);

        pub fn ap_send_message(ptr: *const u8, len: usize);

        pub fn ap_get_time() -> i32;
    }
}


pub struct Platform {
    config: Config,
    gl: GL,
}

impl Platform {
    pub fn new() -> Platform {
        Platform {
            config: Config,
            gl: GL::new(),
        }
    }
}

impl platform::Platform for Platform {
    type GL = GL;
    fn gl(&mut self) -> &mut GL { &mut self.gl
    }

    type Config = Config;
    fn config(&self) -> &Config { &self.config }
    fn config_mut(&mut self) -> &mut Config { &mut self.config }

    fn set_cursor(&mut self, cursor: Cursor) {
        unsafe { ffi::ap_set_cursor(cursor as u8) };
    }

    fn send_message(&mut self, msg: Request) {
        let mut buf = Vec::with_capacity(msg.size());
        msg.write_to(&mut buf)
            .unwrap_or_else(|e| panic!("error encoding msg {:?}: {}", msg, e));
        unsafe {
            ffi::ap_send_message(buf.as_ptr(), buf.len());
        }
    }

    fn get_time(&self) -> i32 {
        unsafe {
            ffi::ap_get_time()
        }
    }
}


// Config

pub struct Config;

impl platform::Config for Config {
    fn get_int(&self, key: ConfigKey) -> i32 {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();

        unsafe {
            ffi::ap_config_get_int(key_bytes.as_ptr(),
                                   key_bytes.len())
        }
    }

    fn set_int(&mut self, key: ConfigKey, value: i32) {
        self.set_str(key, &value.to_string());
    }

    fn get_str(&self, key: ConfigKey) -> String {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();

        unsafe {
            let mut value_len = 0;
            let value_ptr = ffi::ap_config_get(key_bytes.as_ptr(),
                                               key_bytes.len(),
                                               &mut value_len);
            String::from_raw_parts(value_ptr, value_len, value_len)
        }
    }

    fn set_str(&mut self, key: ConfigKey, value: &str) {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();
        let value_bytes = value.as_bytes();

        unsafe {
            ffi::ap_config_set(key_bytes.as_ptr(),
                               key_bytes.len(),
                               value_bytes.as_ptr(),
                               value_bytes.len());
        }
    }

    fn clear(&mut self, key: ConfigKey) {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();

        unsafe {
            ffi::ap_config_clear(key_bytes.as_ptr(),
                                 key_bytes.len());
        }
    }
}
