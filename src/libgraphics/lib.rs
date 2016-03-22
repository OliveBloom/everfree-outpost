#![crate_name = "graphics"]
#![no_std]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
use std::prelude::v1::*;

#[macro_use] extern crate bitflags;
extern crate physics;


pub mod types;
pub mod structures;
pub mod terrain;
pub mod lights;


const ATLAS_SIZE: u16 = 32;

const LOCAL_BITS: usize = 3;
const LOCAL_SIZE: i32 = 1 << LOCAL_BITS;


pub trait IntrusiveCorner {
    fn corner(&self) -> &(u8, u8);
    fn corner_mut(&mut self) -> &mut (u8, u8);
}

pub fn emit_quad<T: Copy+IntrusiveCorner>(buf: &mut [T],
                                          idx: &mut usize,
                                          vertex: T) {
    for &corner in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
        buf[*idx] = vertex;
        *buf[*idx].corner_mut() = corner;
        *idx += 1;
    }
}

pub fn remaining_quads<T>(buf: &[T], idx: usize) -> usize {
    (buf.len() - idx) / 6
}
