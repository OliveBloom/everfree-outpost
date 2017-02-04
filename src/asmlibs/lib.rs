#![crate_name = "asmlibs"]
#![no_std]

#![feature(
    alloc,
    box_syntax,
    filling_drop,
    iter_arith,
    oom,
)]

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

extern crate alloc;
#[macro_use] extern crate bitflags;

extern crate client;
extern crate common_proto;
extern crate common_types;
extern crate physics;

use std::mem;
use std::ptr;
use std::slice;
use common_proto::wire::ReadFrom;

use client::Data;

use physics::v3::V2;

use client::inventory;
use client::graphics::light;
use client::graphics::structure;
use client::graphics::terrain;
use client::ui;

mod platform;
mod gl;
mod io;


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
pub unsafe extern fn handle_message(client: &mut Client,
                                    ptr: *const u8,
                                    len: usize) {
    let buf = make_slice(ptr, len);
    let msg = common_proto::game::Response::read_from(&mut io::Cursor::new(buf))
        .unwrap_or_else(|e| panic!("error parsing message ({:x}): {}", buf[0], e));
    client.handle_message(msg);
}

#[no_mangle]
pub unsafe extern fn client_reset(client: &mut Client) {
    client.reset_all();
}

#[no_mangle]
pub unsafe extern fn client_reset_renderer(client: &mut Client) {
    client.reset_renderer();
}

// Inputs

#[no_mangle]
pub unsafe extern fn input_key_down(client: &mut Client,
                                    code: u8,
                                    shift: u8) -> u8 {
    let mods =
        if shift != 0 { 0x01 } else { 0 };
    client.input_key_down(code, mods) as u8
}

#[no_mangle]
pub unsafe extern fn input_key_up(client: &mut Client,
                                  code: u8,
                                  shift: u8) -> u8 {
    let mods =
        if shift != 0 { 0x01 } else { 0 };
    client.input_key_up(code, mods) as u8
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
                                      y: i32,
                                      button: u8,
                                      mods: u8) -> u8 {
    client.input_mouse_down(V2::new(x, y), button, mods) as u8
}

#[no_mangle]
pub unsafe extern fn input_mouse_up(client: &mut Client,
                                    x: i32,
                                    y: i32,
                                    button: u8,
                                    mods: u8) -> u8 {
    client.input_mouse_up(V2::new(x, y), button, mods) as u8
}


// Graphics

unsafe fn make_slice<T>(ptr: *const T, byte_len: usize) -> &'static [T] {
    slice::from_raw_parts(ptr, byte_len / mem::size_of::<T>())
}

unsafe fn make_boxed_slice<T>(ptr: *mut T, byte_len: usize) -> Box<[T]> {
    assert!(byte_len % mem::size_of::<T>() == 0);
    let raw: *mut [T] = slice::from_raw_parts_mut(ptr, byte_len / mem::size_of::<T>());
    Box::from_raw(raw)
}


#[no_mangle]
pub unsafe extern fn render_frame(client: &mut Client) {
    client.render_frame();
}

// Misc

#[no_mangle]
pub unsafe extern fn debug_record(client: &mut Client,
                                  frame_time: i32) {
    client.debug_record(frame_time);
}

#[no_mangle]
pub unsafe extern fn handle_pong(client: &mut Client,
                                 client_send: i32,
                                 client_recv: i32,
                                 server: u16) {
    client.handle_pong(client_send, client_recv, server);
}

#[no_mangle]
pub unsafe extern fn predict_arrival(client: &mut Client,
                                     extra_delay: i32) -> i32 {
    client.predict_arrival(extra_delay)
}

#[no_mangle]
pub unsafe extern fn calc_scale(client: &Client,
                                size_x: u16,
                                size_y: u16) -> i16 {
    client.calc_scale((size_x, size_y))
}

#[no_mangle]
pub unsafe extern fn resize_window(client: &mut Client,
                                   width: u16,
                                   height: u16) {
    client.resize_window((width, height));
}

#[no_mangle]
pub unsafe extern fn ponyedit_render(client: &mut Client,
                                     appearance: u32) {
    client.ponyedit_render(appearance);
}


#[no_mangle]
pub unsafe extern fn client_bench(client: &mut Client) -> i32 {
    extern "C" {
        fn ap_get_time() -> i32;
    }

    let start = ap_get_time();
    client.bench();
    let end = ap_get_time();

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
