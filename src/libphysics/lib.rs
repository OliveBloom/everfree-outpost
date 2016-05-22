#![crate_name = "physics"]
#![no_std]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
use std::prelude::v1::*;

// TODO: currently this is the way to get the asm.js log macros
#[cfg(asmjs)] #[macro_use] extern crate asmrt;
#[cfg(not(asmjs))] #[macro_use] extern crate log;

#[macro_use] extern crate bitflags;

use std::collections::VecDeque;
use v3::{Vn, V3, V2, Region, scalar};


pub mod v3;
mod walk;
pub mod walk2;


pub const TILE_BITS: usize = 5;
pub const TILE_SIZE: i32 = 1 << TILE_BITS;      // 32
pub const TILE_MASK: i32 = TILE_SIZE - 1;

pub const CHUNK_BITS: usize = 4;
pub const CHUNK_SIZE: i32 = 1 << CHUNK_BITS;    // 16
pub const CHUNK_MASK: i32 = CHUNK_SIZE - 1;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Shape {
    Empty = 0,
    Floor = 1,
    Solid = 2,
    //RampE = 3,
    //RampW = 4,
    //RampS = 5,
    RampN = 6,
}

impl Shape {
    pub fn from_primitive(i: usize) -> Option<Shape> {
        use self::Shape::*;
        let s = match i {
            0 => Empty,
            1 => Floor,
            2 => Solid,
            6 => RampN,
            // TODO: add ramp variants once they are actually supported
            _ => return None,
        };
        Some(s)
    }

    pub fn is_ramp(&self) -> bool {
        use self::Shape::*;
        match *self {
            RampN => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self {
            Shape::Empty => true,
            _ => false,
        }
    }
}


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


pub fn collide<S: ShapeSource>(chunk: &S, pos: V3, size: V3, velocity: V3) -> (V3, i32) {
    use walk::GroundStep;

    if velocity == scalar(0) {
        return (pos, core::i32::MAX);
    }

    let end_pos = walk_path(chunk, pos, size, velocity, GroundStep::new(size));

    // Find the actual velocity after adjustment
    let velocity_mag = velocity.abs().max();
    let offset_mag = (end_pos - pos).abs().max();
    let t =
        if velocity_mag == 0 {
            0
        } else {
            offset_mag * 1000 / velocity_mag
        };

    (end_pos, t)
}


trait StepCallback {
    fn adjust_offset<S: ShapeSource>(&self, chunk: &S, pos: V3, dir: V3) -> V3;
}

fn walk_path<S, CB>(chunk: &S, start_pos: V3, _size: V3, velocity: V3,
                    cb: CB) -> V3
        where S: ShapeSource,
              CB: StepCallback {
    let dir = velocity.signum();
    let mut pos = start_pos;

    let mut last_adj_dir = dir;

    for i in 0..500 {
        // Try up to 4 times to find a direction we can move in.
        let adj_dir = cb.adjust_offset(chunk, pos, dir);

        // Stop if the adjustment changes, sending us in a new direction.  Otherwise, stop if
        // progress is completely blocked.
        if (adj_dir != last_adj_dir && i != 0) || adj_dir == scalar(0) {
            break;
        }

        last_adj_dir = adj_dir;
        pos = pos + adj_dir;
    }

    pos
}


/// Check if the floor is blocked at this location.
fn check_floor<S: ShapeSource>(chunk: &S, pos: V3) -> bool {
    chunk.get_shape(pos) == Shape::Solid &&
    chunk.get_shape(pos + V3::new(0, 0, 1)) == Shape::Solid
}

/// Check if the ceiling is blocked at some point above this location.
fn check_ceiling<S: ShapeSource>(chunk: &S, pos: V3) -> bool {
    let base = pos.reduce();
    for z in pos.z + 2 .. 16 {
        if chunk.get_shape(base.extend(z)) != Shape::Empty {
            return true;
        }
    }
    false
}

/// Check if floodfilling should stop at this point.
fn stop_fill<S: ShapeSource>(chunk: &S, pos: V3) -> bool {
    // Stop if floor is blocked or if we've left the covered area.
    check_floor(chunk, pos) || !check_ceiling(chunk, pos)
}

pub mod fill_flags {
    bitflags! {
        pub flags Flags: u8 {
            /// The cell has already been enqueued.  There's no separate flag for cells that have
            /// been fully processed, since each cell will be enqueued at most once.
            const ENQUEUED =    1 << 0,
            const INSIDE =      1 << 1,
            const INSIDE_NW =   1 << 2,
            const INSIDE_NE =   1 << 3,
            const INSIDE_SW =   1 << 4,
            const INSIDE_SE =   1 << 5,
            const ALL_CORNERS_INSIDE =
                INSIDE_NW.bits | INSIDE_NE.bits | INSIDE_SW.bits | INSIDE_SE.bits,
        }
    }
}

pub fn floodfill<S>(start: V3,
                    bounds: Region<V2>,
                    chunk: &S,
                    grid: &mut [fill_flags::Flags],
                    grid_bounds: Region<V2>)
        where S: ShapeSource {
    let mut queue = VecDeque::new();
    queue.push_back(start.reduce());
    let z = start.z;
    while let Some(pos) = queue.pop_front() {
        if !stop_fill(chunk, pos.extend(z)) {
            grid[grid_bounds.index(pos)].insert(fill_flags::INSIDE);

            let mut maybe_enqueue = |x, y| {
                let p = pos + V2::new(x, y);
                if !bounds.contains(p) {
                    return;
                }
                let idx = grid_bounds.index(p);
                if !grid[idx].contains(fill_flags::ENQUEUED) {
                    grid[idx].insert(fill_flags::ENQUEUED);
                    queue.push_back(p);
                }
            };

            maybe_enqueue(-1,  0);
            maybe_enqueue( 1,  0);
            maybe_enqueue( 0, -1);
            maybe_enqueue( 0,  1);
        }
    }

    // Populate corner-fill flags.
    for pos in grid_bounds.points() {
        let inside = |grid: &[fill_flags::Flags], dx, dy| {
            let p = pos + V2::new(dx, dy);
            bounds.contains(p) &&
                grid[grid_bounds.index(p)].contains(fill_flags::INSIDE)
        };

        if inside(grid, 0, 0) {
            grid[grid_bounds.index(pos)].insert(fill_flags::ALL_CORNERS_INSIDE);
        } else {
            let n = inside(grid,  0, -1);
            let s = inside(grid,  0,  1);
            let w = inside(grid, -1,  0);
            let e = inside(grid,  1,  0);

            let nw = n || w || inside(grid, -1, -1);
            let ne = n || e || inside(grid,  1, -1);
            let sw = s || w || inside(grid, -1,  1);
            let se = s || e || inside(grid,  1,  1);

            grid[grid_bounds.index(pos)].insert(
                if nw { fill_flags::INSIDE_NW } else { fill_flags::Flags::empty() } |
                if ne { fill_flags::INSIDE_NE } else { fill_flags::Flags::empty() } |
                if sw { fill_flags::INSIDE_SW } else { fill_flags::Flags::empty() } |
                if se { fill_flags::INSIDE_SE } else { fill_flags::Flags::empty() });
        }
    }
}
