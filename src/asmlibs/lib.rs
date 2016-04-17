#![crate_name = "asmlibs"]
#![no_std]

#![feature(
    alloc,
    box_syntax,
    filling_drop,
    iter_arith,
    oom,
    raw,
    unsafe_no_drop_flag,
)]

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

extern crate alloc;

extern crate client;
extern crate physics;

use std::mem;
use std::ptr;
use std::raw;
use std::slice;

use client::Data;

use physics::v3::{V3, V2, Region};
use physics::TILE_BITS;

use client::inventory;
use client::graphics::light;
use client::graphics::structure;
use client::graphics::terrain;
use client::graphics::types as gfx_types;
use client::ui;

mod gl;
mod platform;


pub type Client<'d> = client::Client<'d, platform::Platform>;


// New API

// Init

#[no_mangle]
pub unsafe extern fn asmlibs_init() {
    fn oom() -> ! {
        panic!("out of memory");
    }

    alloc::oom::set_oom_handler(oom);
}

#[no_mangle]
pub unsafe extern fn data_init(blobs: &[(*mut u8, usize); 10],
                               out: *mut Data) {
    let blocks =            make_boxed_slice(blobs[0].0 as *mut _, blobs[0].1);
    let item_defs =         make_boxed_slice(blobs[1].0 as *mut _, blobs[1].1);
    let item_strs =         make_boxed_slice(blobs[2].0 as *mut _, blobs[2].1);
    let templates =         make_boxed_slice(blobs[3].0 as *mut _, blobs[3].1);
    let template_parts =    make_boxed_slice(blobs[4].0 as *mut _, blobs[4].1);
    let template_verts =    make_boxed_slice(blobs[5].0 as *mut _, blobs[5].1);
    let template_shapes =   make_boxed_slice(blobs[6].0 as *mut _, blobs[6].1);
    let animations =        make_boxed_slice(blobs[7].0 as *mut _, blobs[7].1);
    let sprite_layers =     make_boxed_slice(blobs[8].0 as *mut _, blobs[8].1);
    let sprite_graphics =   make_boxed_slice(blobs[9].0 as *mut _, blobs[9].1);

    let item_strs = String::from_utf8(item_strs.into_vec()).unwrap().into_boxed_str();

    ptr::write(out, Data::new(
            blocks, item_defs, item_strs,
            templates, template_parts, template_verts, template_shapes,
            animations, sprite_layers, sprite_graphics,
            ));
}

#[no_mangle]
pub unsafe extern fn client_init(data_ptr: *const Data,
                                 out: *mut Client) {
    ptr::write(out, Client::new(&*data_ptr, platform::Platform::new()));
}

#[no_mangle]
pub unsafe extern fn client_reset(client: &mut Client) {
    client.reset_all();
}

// Chunks

#[no_mangle]
pub unsafe extern fn load_terrain_chunk(client: &mut Client,
                                        cx: i32,
                                        cy: i32,
                                        data_ptr: *const u16,
                                        data_byte_len: usize) {
    let data = make_slice(data_ptr, data_byte_len);
    client.load_terrain_chunk(V2::new(cx, cy), data);
}

// Structures

#[no_mangle]
pub unsafe extern fn structure_appear(client: &mut Client,
                                      id: u32,
                                      pos_x: u8,
                                      pos_y: u8,
                                      pos_z: u8,
                                      template_id: u32,
                                      oneshot_start: u16) {
    client.structure_appear(id,
                            (pos_x, pos_y, pos_z),
                            template_id,
                            oneshot_start);
}

#[no_mangle]
pub unsafe extern fn structure_gone(client: &mut Client,
                                    id: u32) {
    client.structure_gone(id);
}

#[no_mangle]
pub unsafe extern fn structure_replace(client: &mut Client,
                                       id: u32,
                                       template_id: u32,
                                       oneshot_start: u16) {
    client.structure_replace(id, template_id, oneshot_start);
}

// Entities

#[no_mangle]
pub unsafe extern fn entity_appear(client: &mut Client,
                                   id: u32,
                                   appearance: u32,
                                   name_ptr: *mut u8,
                                   name_len: usize) {
    let name =
        if name_ptr.is_null() {
            None
        } else {
            let name_bytes = make_boxed_slice(name_ptr, name_len).into_vec();
            Some(String::from_utf8(name_bytes).unwrap())
        };
    client.entity_appear(id, appearance, name);
}

#[no_mangle]
pub unsafe extern fn entity_gone(client: &mut Client,
                                 id: u32) {
    client.entity_gone(id);
}

#[no_mangle]
pub unsafe extern fn entity_update(client: &mut Client,
                                   id: u32,
                                   when: i32,
                                   motion: &client::entity::Motion) {
    client.entity_update(id, when, motion.clone());
}

// Inventories

#[no_mangle]
pub unsafe extern fn inventory_appear(client: &mut Client,
                                      id: u32,
                                      items_ptr: *mut u8,
                                      items_byte_len: usize) {
    let items = make_boxed_slice(items_ptr as *mut inventory::Item, items_byte_len);
    client.inventory_appear(id, items);
}

#[no_mangle]
pub unsafe extern fn inventory_gone(client: &mut Client,
                                    id: u32) {
    client.inventory_gone(id);
}

#[no_mangle]
pub unsafe extern fn inventory_update(client: &mut Client,
                                      inv_id: u32,
                                      slot: usize,
                                      item_id: u16,
                                      quantity: u8) {
    client.inventory_update(inv_id, slot, inventory::Item::new(item_id, quantity));
}

#[no_mangle]
pub unsafe extern fn inventory_main_id(client: &mut Client,
                                       inv_id: u32) {
    client.set_main_inventory_id(inv_id);
}

#[no_mangle]
pub unsafe extern fn inventory_ability_id(client: &mut Client,
                                          inv_id: u32) {
    client.set_ability_inventory_id(inv_id);
}

#[no_mangle]
pub unsafe extern fn input_key(client: &mut Client, code: u8) -> u8 {
    client.input_key(code) as u8
}

#[no_mangle]
pub unsafe extern fn input_mouse_move(client: &mut Client,
                                      x: i32,
                                      y: i32) -> u8 {
    client.input_mouse_move(V2::new(x, y)) as u8
}

#[no_mangle]
pub unsafe extern fn open_inventory_dialog(client: &mut Client) {
    client.open_inventory_dialog()
}




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


#[no_mangle]
pub unsafe extern fn prepare_geometry(client: &mut Client,
                                      cx0: i32,
                                      cy0: i32,
                                      cx1: i32,
                                      cy1: i32,
                                      now: i32) {
    client.prepare_geometry(Region::new(V2::new(cx0, cy0),
                                        V2::new(cx1, cy1)),
                            now);
}

#[no_mangle]
pub unsafe extern fn get_terrain_geometry_buffer(client: &Client,
                                                 len: &mut usize) -> u32 {
    let buf = client.get_terrain_geometry_buffer();
    *len = buf.len();
    buf.name().raw
}

#[no_mangle]
pub unsafe extern fn get_structure_geometry_buffer(client: &Client,
                                                   len: &mut usize) -> u32 {
    let buf = client.get_structure_geometry_buffer();
    *len = buf.len();
    buf.name().raw
}

#[no_mangle]
pub unsafe extern fn get_light_geometry_buffer(client: &Client,
                                               len: &mut usize) -> u32 {
    let buf = client.get_light_geometry_buffer();
    *len = buf.len();
    buf.name().raw
}

#[no_mangle]
pub unsafe extern fn get_ui_geometry_buffer(client: &Client,
                                            len: &mut usize) -> u32 {
    let buf = client.get_ui_geometry_buffer();
    *len = buf.len();
    buf.name().raw
}

#[no_mangle]
pub unsafe extern fn test_render(client: &mut Client,
                                 opcode: u32,
                                 scene: &client::graphics::renderer::Scene,
                                 tex_name: u32) {
    client.test_render(opcode, scene, tex_name);
}


#[no_mangle]
pub unsafe extern fn client_bench(client: &mut Client) -> u32 {
    extern "C" {
        fn now() -> u32;
    }

    let start = now();
    client.bench();
    let end = now();

    end - start
}


// SIZEOF

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Sizes {
    client: usize,
    client_alignment: usize,
    data: usize,
    data_alignment: usize,

    terrain_vertex: usize,
    structure_vertex: usize,
    light_vertex: usize,
    ui_vertex: usize,

    item: usize,
}

#[export_name = "get_sizes"]
pub extern fn get_sizes(sizes: &mut Sizes) -> usize {
    use core::mem::size_of;
    use core::mem::align_of;

    sizes.client = size_of::<Client>();
    sizes.client_alignment = align_of::<Client>();
    sizes.data = size_of::<client::Data>();
    sizes.data_alignment = align_of::<client::Data>();

    sizes.terrain_vertex = size_of::<terrain::Vertex>();
    sizes.structure_vertex = size_of::<structure::Vertex>();
    sizes.light_vertex = size_of::<light::Vertex>();
    sizes.ui_vertex = size_of::<ui::geom::Vertex>();

    sizes.item = size_of::<inventory::Item>();

    size_of::<Sizes>() / size_of::<usize>()
}
