#![crate_name = "client"]
#![no_std]

#![feature(
    // Have to use box syntax because Box::new() in Client::new() tries to allocate 1.5MB of
    // temporary arrays on the stack.
    box_syntax,
    btree_range,
    collections,
    collections_bound,
    core_intrinsics,
    fnbox,
    )]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
#[allow(unused_imports)] use std::prelude::v1::*;

#[macro_use] extern crate bitflags;

extern crate client_fonts;
extern crate client_ui_atlas;
extern crate physics;

pub use client::Client;
pub use data::Data;

#[macro_use] pub mod platform;

pub mod client;
pub mod data;
mod fonts;

mod util;

mod terrain;
mod structures;
pub mod entity;
pub mod inventory;
mod misc;

pub mod graphics;
pub mod ui;

// TODO: change this to u32 (requires adjustment of server timing so that `now` is not negative
// just after startup)
pub type Time = i32;
