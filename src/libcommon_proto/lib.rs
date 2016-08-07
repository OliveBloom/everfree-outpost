#![crate_name = "common_proto"]

#![cfg_attr(asmjs, no_std)]
#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;

extern crate common_types;
#[macro_use] extern crate common_util;
extern crate physics;

pub mod types;
pub mod wire;
pub mod extra_arg;

pub mod game;
