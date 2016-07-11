#![crate_name = "common"]
#![no_std]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
//use std::prelude::v1::*;

// TODO: currently this is the way to get the asm.js log macros
#[cfg(asmjs)] #[macro_use] extern crate asmrt;
#[cfg(not(asmjs))] #[macro_use] extern crate log;


pub mod gauge;
pub use self::gauge::Gauge;

#[cfg(asmjs)] pub mod types_client;
#[cfg(asmjs)] pub use self::types_client as types;

#[cfg(not(asmjs))] pub mod types_server;
#[cfg(not(asmjs))] pub use self::types_server as types;
