#![no_std]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
#[cfg(not(asmjs))] #[macro_use] extern crate log;

pub mod context;
pub mod event;
pub mod geom;
pub mod param;
pub mod widget;
pub mod widgets;
