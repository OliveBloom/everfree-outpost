#![crate_name = "uvedit_asm"]
#![no_std]

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

extern crate physics;


#[no_mangle]
pub extern fn init() {
    println!("test test");
}
