#![crate_name = "python"]

#![feature(
    filling_drop,
    nonzero,
    plugin,
    unsafe_no_drop_flag,
)]

#![plugin(syntax_exts)]

extern crate core;
extern crate libc;
#[macro_use] extern crate log;
extern crate python3_sys;

extern crate physics;
extern crate server_config;
extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;


#[macro_use] pub mod macros;

pub mod api;
pub mod conv;
pub mod exc;
pub mod ptr;
pub mod util;

#[macro_use] pub mod class;
#[macro_use] pub mod rust_ref;
#[macro_use] pub mod rust_val;

mod conv_impls;

pub fn init_builtin_types(module: api::PyRef) {
    conv_impls::init(module);
}
