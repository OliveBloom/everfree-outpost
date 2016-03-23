#![crate_name = "client"]
#![no_std]

#![feature(
    // Have to use box syntax because Box::new() in Client::new() tries to allocate 1.5MB of
    // temporary arrays on the stack.
    box_syntax,
    btree_range,
    collections,
    collections_bound,
    )]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
use std::prelude::v1::*;

#[macro_use] extern crate bitflags;

extern crate physics;

pub use client::Client;
pub use data::Data;

pub mod client;
pub mod data;

mod terrain;
mod structures;
pub mod graphics;
