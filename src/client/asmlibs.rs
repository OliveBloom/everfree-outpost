#![crate_name = "asmlibs"]
#![no_std]

#![feature(raw, alloc, oom, heap_api, core_intrinsics)]

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

extern crate alloc;

extern crate client;
extern crate physics;
extern crate graphics;

use std::intrinsics;
use std::iter;
use std::mem;
use std::ptr;
use std::raw;
use std::slice;

use client::Client;
use client::Data;

use physics::v3::{V3, V2, scalar, Region};
use physics::{Shape, ShapeSource};
use physics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK, TILE_BITS};

use graphics::lights;
use graphics::structures;
use graphics::terrain;
use graphics::types as gfx_types;


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

#[export_name = "collide"]
pub extern fn collide_wrapper(client: &Client,
                              input: &CollideArgs,
                              output: &mut CollideResult) {
    let (pos, time) = client.collide(input.pos, input.size, input.velocity);
    output.pos = pos;
    output.time = time;
}

#[export_name = "set_region_shape"]
pub extern fn set_region_shape(client: &mut Client,
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

    client.set_region_shape(*bounds, layer, shape);
}

#[export_name = "find_ceiling"]
pub extern fn find_ceiling(client: &Client,
                           pos: &V3) -> i32 {
    client.find_ceiling(*pos)
}

#[export_name = "floodfill"]
pub unsafe extern fn floodfill(client: &Client,
                               pos: &V3,
                               radius: u8,
                               grid_ptr: *mut u8,
                               grid_byte_len: usize,
                               queue_ptr: *mut (u8, u8),
                               queue_byte_len: usize) {
    let grid = make_slice_mut(grid_ptr as *mut physics::fill_flags::Flags, grid_byte_len);
    let queue = make_slice_mut(queue_ptr, queue_byte_len);
    client.floodfill(*pos, radius, grid, queue);
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


#[export_name = "asmlibs_init"]
pub unsafe extern fn asmlibs_init() {
    fn oom() -> ! {
        panic!("out of memory");
    }

    alloc::oom::set_oom_handler(oom);
}

#[export_name = "data_init"]
pub unsafe extern fn data_init(blobs: &[(*mut u8, usize)],
                               out: *mut Data) {
    let blocks =            make_boxed_slice(blobs[0].0 as *mut _, blobs[0].1);
    let templates =         make_boxed_slice(blobs[1].0 as *mut _, blobs[1].1);
    let template_parts =    make_boxed_slice(blobs[2].0 as *mut _, blobs[2].1);
    let template_verts =    make_boxed_slice(blobs[3].0 as *mut _, blobs[3].1);
    ptr::write(out, Data::new(
            blocks, templates, template_parts, template_verts));
}

#[export_name = "client_init"]
pub unsafe extern fn client_init(data_ptr: *const Data,
                                 out: *mut Client) {
    let data = ptr::read(data_ptr);
    ptr::write(out, Client::new(data));
}

#[export_name = "terrain_geom_reset"]
pub extern fn terrain_geom_reset(client: &mut Client,
                                 cx: i32,
                                 cy: i32) {
    client.terrain_geom_reset(V2::new(cx, cy))
}

#[export_name = "terrain_geom_generate"]
pub unsafe extern fn terrain_geom_generate(client: &mut Client,
                                           buf_ptr: *mut terrain::Vertex,
                                           buf_byte_len: usize,
                                           result: &mut GeometryResult) {
    let buf = make_slice_mut(buf_ptr, buf_byte_len);
    let (count, more) = client.terrain_geom_generate(buf);
    result.vertex_count = count;
    result.more = more as u8;
}


#[export_name = "structure_buffer_init"]
pub unsafe extern fn structure_buffer_init(buf: &mut structures::Buffer,
                                           storage_ptr: *mut structures::Structure,
                                           storage_byte_len: usize) {
}

#[export_name = "structure_buffer_insert"]
pub extern fn structure_buffer_insert(buf: &mut structures::Buffer,
                                      external_id: u32,
                                      pos_x: u8,
                                      pos_y: u8,
                                      pos_z: u8,
                                      template_id: u32,
                                      oneshot_start: u16) -> usize {
    0
}

#[export_name = "structure_buffer_remove"]
pub extern fn structure_buffer_remove(buf: &mut structures::Buffer,
                                      idx: usize) -> u32 {
    buf.remove(idx)
}


#[export_name = "structure_geom_init"]
pub unsafe extern fn structure_geom_init(geom: &mut structures::GeomGen<'static>,
                                         buffer: &'static structures::Buffer,
                                         templates_ptr: *const gfx_types::StructureTemplate,
                                         templates_byte_len: usize,
                                         parts_ptr: *const gfx_types::TemplatePart,
                                         parts_byte_len: usize,
                                         verts_ptr: *const gfx_types::TemplateVertex,
                                         verts_byte_len: usize) {
}

#[export_name = "structure_geom_reset"]
pub extern fn structure_geom_reset(geom: &mut structures::GeomGen,
                                   cx0: i32,
                                   cy0: i32,
                                   cx1: i32,
                                   cy1: i32,
                                   sheet: u8) {
}

#[export_name = "structure_geom_generate"]
pub unsafe extern fn structure_geom_generate(geom: &mut structures::GeomGen,
                                             buf_ptr: *mut structures::Vertex,
                                             buf_byte_len: usize,
                                             result: &mut GeometryResult) {
}


#[export_name = "light_geom_init"]
pub unsafe extern fn light_geom_init(geom: &mut lights::GeomGen<'static>,
                                     buffer: &'static structures::Buffer,
                                     templates_ptr: *const gfx_types::StructureTemplate,
                                     templates_byte_len: usize) {
}

#[export_name = "light_geom_reset"]
pub extern fn light_geom_reset(geom: &mut lights::GeomGen,
                               cx0: i32,
                               cy0: i32,
                               cx1: i32,
                               cy1: i32) {
}

#[export_name = "light_geom_generate"]
pub unsafe extern fn light_geom_generate(geom: &mut lights::GeomGen,
                                         buf_ptr: *mut lights::Vertex,
                                         buf_byte_len: usize,
                                         result: &mut GeometryResult) {
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
    structure: usize,

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
    sizes.structure = size_of::<structures::Structure>();

    sizes.terrain_vertex = size_of::<terrain::Vertex>();
    sizes.structure_vertex = size_of::<structures::Vertex>();
    sizes.light_vertex = size_of::<lights::Vertex>();

    size_of::<Sizes>() / size_of::<usize>()
}
