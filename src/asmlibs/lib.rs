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
#[macro_use] extern crate bitflags;

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
pub unsafe extern fn data_init(blob_ptr: *mut u8,
                               blob_len: usize,
                               out: *mut Data) {
    let blob = make_boxed_slice(blob_ptr, blob_len);
    ptr::write(out, Data::new(blob));
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

#[no_mangle]
pub unsafe extern fn set_pawn_id(client: &mut Client,
                                 pawn_id: u32) {
    if pawn_id == -1_i32 as u32 {
        client.clear_pawn_id();
    } else {
        client.set_pawn_id(pawn_id);
    }
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
pub unsafe extern fn input_mouse_down(client: &mut Client,
                                      x: i32,
                                      y: i32) -> u8 {
    client.input_mouse_down(V2::new(x, y)) as u8
}

#[no_mangle]
pub unsafe extern fn input_mouse_up(client: &mut Client,
                                    x: i32,
                                    y: i32) -> u8 {
    client.input_mouse_up(V2::new(x, y)) as u8
}

#[no_mangle]
pub unsafe extern fn open_inventory_dialog(client: &mut Client) {
    client.open_inventory_dialog()
}

#[no_mangle]
pub unsafe extern fn get_active_item(client: &mut Client) -> u16 {
    client.get_active_item()
}

#[no_mangle]
pub unsafe extern fn get_active_ability(client: &mut Client) -> u16 {
    client.get_active_ability()
}



// Physics

#[no_mangle]
pub extern fn feed_input(client: &mut Client,
                         time: i32,
                         dir_x: i32,
                         dir_y: i32,
                         dir_z: i32) {
    client.feed_input(time, V3::new(dir_x, dir_y, dir_z));
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
pub unsafe extern fn render_frame(client: &mut Client,
                                  now: i32,
                                  future: i32) {
    client.render_frame(now, future);
}

// Misc

#[no_mangle]
pub unsafe extern fn debug_record(client: &mut Client,
                                  frame_time: i32,
                                  ping: u32) {
    client.debug_record(frame_time, ping);
}

#[no_mangle]
pub unsafe extern fn init_day_night(client: &mut Client,
                                    base_time: i32,
                                    cycle_ms: i32) {
    client.init_day_night(base_time, cycle_ms);
}

#[no_mangle]
pub unsafe extern fn resize_window(client: &mut Client,
                                   width: u16,
                                   height: u16) {
    client.resize_window((width, height));
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

    scene: usize,

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

    sizes.scene = size_of::<client::graphics::renderer::Scene>();

    sizes.item = size_of::<inventory::Item>();

    size_of::<Sizes>() / size_of::<usize>()
}
