#![crate_name = "asmlibs"]
#![no_std]

#![feature(raw, alloc, oom, heap_api, core_intrinsics)]

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

extern crate alloc;

extern crate client;
extern crate physics;

use std::intrinsics;
use std::iter;
use std::mem;
use std::ptr;
use std::raw;
use std::slice;

use client::Client;
use client::Data;

use physics::v3::{V3, V2, Vn, scalar, Region};
use physics::{Shape, ShapeSource};
use physics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK, TILE_SIZE, TILE_BITS};

use client::graphics::lights;
use client::graphics::structures;
use client::graphics::terrain;
use client::graphics::types as gfx_types;


// Physics

#[derive(Clone, Copy)]
pub struct CollideArgs {
    pub pos: V3,
    pub size: V3,
    pub velocity: V3,
}

#[derive(Clone, Copy)]
pub struct CollideResult {
    pub pos: V3,
    pub time: i32,
}

#[no_mangle]
pub extern fn collide(client: &Client,
                      input: &CollideArgs,
                      output: &mut CollideResult) {
    let (pos, time) = client.collide(input.pos, input.size, input.velocity);
    output.pos = pos;
    output.time = time;
}

#[no_mangle]
pub extern fn set_region_shape(client: &mut Client,
                               bounds: &(V3, V3),
                               layer: usize,
                               shape_data: *const Shape,
                               shape_len: usize) {
    let bounds = Region::new(bounds.0, bounds.0 + bounds.1);
    let shape: &[Shape] = unsafe {
        mem::transmute(raw::Slice {
            data: shape_data,
            len: shape_len,
        })
    };

    client.set_region_shape(bounds, layer, shape);
}

#[no_mangle]
pub extern fn find_ceiling(client: &Client,
                           pos: &V3) -> i32 {
    client.find_ceiling(*pos >> TILE_BITS)
}

#[no_mangle]
pub unsafe extern fn floodfill(client: &Client,
                               pos: &V3,
                               radius: u8,
                               grid_ptr: *mut u8,
                               grid_byte_len: usize,
                               queue_ptr: *mut (u8, u8),
                               queue_byte_len: usize) {
    let grid = make_slice_mut(grid_ptr as *mut physics::fill_flags::Flags, grid_byte_len);
    let queue = make_slice_mut(queue_ptr, queue_byte_len);
    client.floodfill(*pos >> TILE_BITS, radius, grid, queue);
}


// Graphics

unsafe fn make_slice<T>(ptr: *const T, byte_len: usize) -> &'static [T] {
    slice::from_raw_parts(ptr, byte_len / mem::size_of::<T>())
}

unsafe fn make_slice_mut<T>(ptr: *mut T, byte_len: usize) -> &'static mut [T] {
    slice::from_raw_parts_mut(ptr, byte_len / mem::size_of::<T>())
}

unsafe fn make_boxed_slice<T>(ptr: *mut T, byte_len: usize) -> Box<[T]> {
    assert!(byte_len % mem::size_of::<T>() == 0);
    let raw: *mut [T] = mem::transmute(raw::Slice {
        data: ptr,
        len: byte_len / mem::size_of::<T>(),
    });
    Box::from_raw(raw)
}

pub struct GeometryResult {
    vertex_count: usize,
    more: u8,
}


#[no_mangle]
pub unsafe extern fn asmlibs_init() {
    fn oom() -> ! {
        panic!("out of memory");
    }

    alloc::oom::set_oom_handler(oom);
}

#[no_mangle]
pub unsafe extern fn data_init(blobs: &[(*mut u8, usize); 5],
                               out: *mut Data) {
    let blocks =            make_boxed_slice(blobs[0].0 as *mut _, blobs[0].1);
    let templates =         make_boxed_slice(blobs[1].0 as *mut _, blobs[1].1);
    let template_parts =    make_boxed_slice(blobs[2].0 as *mut _, blobs[2].1);
    let template_verts =    make_boxed_slice(blobs[3].0 as *mut _, blobs[3].1);
    let template_shapes =   make_boxed_slice(blobs[4].0 as *mut _, blobs[4].1);
    ptr::write(out, Data::new(
            blocks, templates, template_parts, template_verts, template_shapes));
}

#[no_mangle]
pub unsafe extern fn client_init(data_ptr: *const Data,
                                 out: *mut Client) {
    let data = ptr::read(data_ptr);
    ptr::write(out, Client::new(data));
}

#[no_mangle]
pub unsafe extern fn load_terrain_chunk(client: &mut Client,
                                 cx: i32,
                                 cy: i32,
                                 blocks_ptr: *const u16,
                                 blocks_byte_len: usize) {
    assert!(blocks_byte_len == mem::size_of::<gfx_types::BlockChunk>());
    let blocks = &*(blocks_ptr as *const gfx_types::BlockChunk);
    client.load_terrain_chunk(V2::new(cx, cy), blocks);
}

#[no_mangle]
pub extern fn terrain_geom_reset(client: &mut Client,
                                 cx: i32,
                                 cy: i32) {
    client.terrain_geom_reset(V2::new(cx, cy))
}

#[no_mangle]
pub unsafe extern fn terrain_geom_generate(client: &mut Client,
                                           buf_ptr: *mut terrain::Vertex,
                                           buf_byte_len: usize,
                                           result: &mut GeometryResult) {
    let buf = make_slice_mut(buf_ptr, buf_byte_len);
    let (count, more) = client.terrain_geom_generate(buf);
    result.vertex_count = count;
    result.more = more as u8;
}


#[no_mangle]
pub extern fn structure_buffer_insert(client: &mut Client,
                                      external_id: u32,
                                      pos_x: u8,
                                      pos_y: u8,
                                      pos_z: u8,
                                      template_id: u32,
                                      oneshot_start: u16) -> usize {
    client.structure_buffer_insert(external_id,
                                   (pos_x, pos_y, pos_z),
                                   template_id,
                                   oneshot_start)
}

#[no_mangle]
pub extern fn structure_buffer_remove(client: &mut Client,
                                      idx: usize) -> u32 {
    client.structure_buffer_remove(idx)
}


#[no_mangle]
pub extern fn structure_geom_reset(client: &mut Client,
                                   cx0: i32,
                                   cy0: i32,
                                   cx1: i32,
                                   cy1: i32,
                                   sheet: u8) {
    client.structure_geom_reset(Region::new(V2::new(cx0, cy0), V2::new(cx1, cy1)), sheet);
}

#[no_mangle]
pub unsafe extern fn structure_geom_generate(client: &mut Client,
                                             buf_ptr: *mut structures::Vertex,
                                             buf_byte_len: usize,
                                             result: &mut GeometryResult) {
    let buf = make_slice_mut(buf_ptr, buf_byte_len);
    let (count, more) = client.structure_geom_generate(buf);
    result.vertex_count = count;
    result.more = more as u8;
}

#[no_mangle]
pub extern fn light_geom_reset(client: &mut Client,
                               cx0: i32,
                               cy0: i32,
                               cx1: i32,
                               cy1: i32) {
    client.light_geom_reset(Region::new(V2::new(cx0, cy0), V2::new(cx1, cy1)));
}

#[no_mangle]
pub unsafe extern fn light_geom_generate(client: &mut Client,
                                         buf_ptr: *mut lights::Vertex,
                                         buf_byte_len: usize,
                                         result: &mut GeometryResult) {
    let buf = make_slice_mut(buf_ptr, buf_byte_len);
    let (count, more) = client.light_geom_generate(buf);
    result.vertex_count = count;
    result.more = more as u8;
}


// SIZEOF

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Sizes {
    client: usize,
    client_alignment: usize,
    data: usize,

    block_data: usize,
    structures_template: usize,
    template_part: usize,
    template_vertex: usize,

    block_chunk: usize,

    terrain_vertex: usize,
    structure_vertex: usize,
    light_vertex: usize,
}

#[export_name = "get_sizes"]
pub extern fn get_sizes(sizes: &mut Sizes) -> usize {
    use core::mem::size_of;
    use core::mem::align_of;

    sizes.client = size_of::<Client>();
    sizes.client_alignment = align_of::<Client>();
    sizes.data = size_of::<client::Data>();

    sizes.block_data = size_of::<gfx_types::BlockData>();
    sizes.structures_template = size_of::<gfx_types::StructureTemplate>();
    sizes.template_part = size_of::<gfx_types::TemplatePart>();
    sizes.template_vertex = size_of::<gfx_types::TemplateVertex>();

    sizes.block_chunk = size_of::<gfx_types::BlockChunk>();

    sizes.terrain_vertex = size_of::<terrain::Vertex>();
    sizes.structure_vertex = size_of::<structures::Vertex>();
    sizes.light_vertex = size_of::<lights::Vertex>();

    size_of::<Sizes>() / size_of::<usize>()
}
