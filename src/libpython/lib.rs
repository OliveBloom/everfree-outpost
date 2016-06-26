#![crate_name = "python"]

#![feature(
    filling_drop,
    nonzero,
    unsafe_no_drop_flag,
)]

extern crate core;
#[macro_use] extern crate log;
extern crate python3_sys;

#[macro_use] extern crate server_util;


#[macro_use] pub mod macros;

pub mod api;
pub mod exc;
pub mod ptr;
pub mod util;
