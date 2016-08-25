#![crate_name = "physics"]
#![no_std]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
use std::prelude::v1::*;

// TODO: currently this is the way to get the asm.js log macros
#[cfg(asmjs)] #[macro_use] extern crate asmrt;
#[cfg(not(asmjs))] #[macro_use] extern crate log;

#[macro_use] extern crate bitflags;
extern crate common_types;

pub use common_types::Shape;

use v3::V3;


pub mod v3;
pub mod walk2;
pub mod floodfill;


pub const TILE_BITS: usize = 5;
pub const TILE_SIZE: i32 = 1 << TILE_BITS;      // 32
pub const TILE_MASK: i32 = TILE_SIZE - 1;

pub const CHUNK_BITS: usize = 4;
pub const CHUNK_SIZE: i32 = 1 << CHUNK_BITS;    // 16
pub const CHUNK_MASK: i32 = CHUNK_SIZE - 1;

// Doesn't quite belong here, but it's good enough
pub const LOCAL_BITS: usize = 3;
pub const LOCAL_SIZE: i32 = 1 << LOCAL_BITS;
pub const LOCAL_MASK: i32 = LOCAL_SIZE - 1;


pub trait ShapeSource {
    fn get_shape(&self, pos: V3) -> Shape;

    fn get_shape_below(&self, mut pos: V3) -> (Shape, i32) {
        while pos.z >= 0 {
            let s = self.get_shape(pos);
            if !s.is_empty() {
                return (s, pos.z);
            }
            pos.z -= 1;
        }
        (Shape::Empty, 0)
    }
}
