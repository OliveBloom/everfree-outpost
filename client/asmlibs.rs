#![crate_name = "asmlibs"]
#![no_std]

#![feature(no_std)]
#![feature(core)]
#![feature(static_assert)]

extern crate core;
extern crate physics;
extern crate graphics;
#[macro_use] extern crate asmrt;

use core::prelude::*;
use core::mem;
use core::raw;
use physics::v3::{V3, scalar, Region};
use physics::{Shape, ShapeSource};
use physics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK};
use graphics::{BlockData, BlockChunk, LocalChunks};
use graphics::{TerrainVertex, TerrainGeometryBuffer};

mod std {
    pub use core::fmt;
    pub use core::marker;
}


pub const LOCAL_SIZE: i32 = 8;
pub const LOCAL_BITS: usize = 3;
pub const LOCAL_MASK: i32 = LOCAL_SIZE - 1;
#[allow(dead_code)] #[static_assert]
static LOCAL_SIZE_BITS: bool = LOCAL_SIZE == 1 << LOCAL_BITS as usize;

pub const REPEAT_SIZE: i32 = 2;
pub const REPEAT_BITS: i32 = 1;
pub const REPEAT_MASK: i32 = REPEAT_SIZE - 1;
#[allow(dead_code)] #[static_assert]
static REPEAT_SIZE_BITS: bool = REPEAT_SIZE == 1 << REPEAT_BITS as usize;

pub const NUM_LAYERS: usize = 2;


// Physics

pub type ShapeChunk = [Shape; 1 << (3 * CHUNK_BITS)];

pub struct ShapeLayers {
    base: ShapeChunk,
    layers: [ShapeChunk; NUM_LAYERS],
    merged: ShapeChunk,
}

impl ShapeLayers {
    fn refresh(&mut self, bounds: Region) {
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));

        for p in bounds.intersect(chunk_bounds).points() {
            let idx = chunk_bounds.index(p);
            self.merged[idx] = self.base[idx];

            for layer in self.layers.iter() {
                if shape_overrides(self.merged[idx], layer[idx]) {
                    self.merged[idx] = layer[idx];
                }
            }
        }
    }
}

fn shape_overrides(old: Shape, new: Shape) -> bool {
    match (old, new) {
        (Shape::Empty, _) => true,

        (Shape::Floor, Shape::Empty) => false,
        (Shape::Floor, _) => true,

        (Shape::Solid, _) => false,

        _ => false,
    }
}

struct AsmJsShapeSource<'a> {
    layers: &'a [ShapeLayers; 1 << (2 * LOCAL_BITS)],
}

impl<'a> ShapeSource for AsmJsShapeSource<'a> {
    fn get_shape(&self, pos: V3) -> Shape {
        let V3 { x: tile_x, y: tile_y, z: tile_z } = pos & scalar(CHUNK_MASK);
        let V3 { x: chunk_x, y: chunk_y, z: _ } = (pos >> CHUNK_BITS) & scalar(LOCAL_MASK);

        let chunk_idx = chunk_y * LOCAL_SIZE + chunk_x;
        let tile_idx = (tile_z * CHUNK_SIZE + tile_y) * CHUNK_SIZE + tile_x;

        let shape = self.layers[chunk_idx as usize].merged[tile_idx as usize];
        shape
    }
}


#[derive(Copy)]
pub struct CollideArgs {
    pub pos: V3,
    pub size: V3,
    pub velocity: V3,
}

#[derive(Copy)]
pub struct CollideResult {
    pub pos: V3,
    pub time: i32,
}

#[export_name = "collide"]
pub extern fn collide_wrapper(layers: &[ShapeLayers; 1 << (2 * LOCAL_BITS)],
                              input: &CollideArgs,
                              output: &mut CollideResult) {
    let (pos, time) = physics::collide(&AsmJsShapeSource { layers: layers },
                                       input.pos, input.size, input.velocity);
    output.pos = pos;
    output.time = time;
}

#[export_name = "set_region_shape"]
pub extern fn set_region_shape(layers: &mut [ShapeLayers; 1 << (2 * LOCAL_BITS)],
                               bounds: &Region,
                               layer: usize,
                               shape_data: *const Shape,
                               shape_len: usize) {
    let shape: &[Shape] = unsafe {
        mem::transmute(raw::Slice {
            data: shape_data,
            len: shape_len,
        })
    };

    let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
    for p in bounds.points() {
        // div_floor requires an extra LLVM intrinsic.
        let cpos = p.reduce() >> CHUNK_BITS;
        let masked_cpos = cpos & scalar(LOCAL_MASK);
        let cidx = masked_cpos.y * LOCAL_SIZE + masked_cpos.x;

        let offset = p & scalar(CHUNK_MASK);
        let out_idx = chunk_bounds.index(offset);
        let in_idx = bounds.index(p);

        layers[cidx as usize].layers[layer][out_idx] = shape[in_idx];
    }
}

#[export_name = "refresh_shape_cache"]
pub extern fn refresh_shape_cache(layers: &mut [ShapeLayers; 1 << (2 * LOCAL_BITS)],
                                  bounds: &Region) {
    let chunk_bounds = bounds.reduce().div_round(CHUNK_SIZE);

    for cpos in chunk_bounds.points() {
        let masked_cpos = cpos & scalar(LOCAL_MASK);
        let cidx = masked_cpos.y * LOCAL_SIZE + masked_cpos.x;

        let base = cpos.extend(0) * scalar(CHUNK_SIZE);
        layers[cidx as usize].refresh(*bounds - base);
    }
}


// Graphics

#[export_name = "load_chunk"]
pub extern fn load_chunk(local: &mut LocalChunks,
                         chunk: &BlockChunk,
                         cx: u16,
                         cy: u16) {
    graphics::load_chunk(local, chunk, cx, cy);
}

#[export_name = "generate_geometry"]
pub extern fn generate_geometry(local: &LocalChunks,
                                block_data: &BlockData,
                                geom: &mut TerrainGeometryBuffer,
                                cx: u16,
                                cy: u16) -> usize {
    graphics::generate_geometry(local, block_data, geom, cx, cy)
}


// SIZEOF

#[repr(C)]
#[derive(Copy, Debug)]
pub struct Sizes {
    shape_chunk: usize,
    shape_layers: usize,

    block_data: usize,
    block_chunk: usize,
    local_chunks: usize,

    terrain_vertex: usize,
    terrain_geometry_buffer: usize,
}

#[export_name = "get_sizes"]
pub extern fn get_sizes(sizes: &mut Sizes, num_sizes: &mut usize) {
    use core::mem::size_of;

    sizes.shape_chunk = size_of::<ShapeChunk>();
    sizes.shape_layers = size_of::<ShapeLayers>();

    sizes.block_data = size_of::<BlockData>();
    sizes.block_chunk = size_of::<BlockChunk>();
    sizes.local_chunks = size_of::<LocalChunks>();

    sizes.terrain_vertex = size_of::<TerrainVertex>();
    sizes.terrain_geometry_buffer = size_of::<TerrainGeometryBuffer>();

    *num_sizes = size_of::<Sizes>() / size_of::<usize>();
}


#[export_name = "test"]
pub extern fn test_wrapper() {
}
