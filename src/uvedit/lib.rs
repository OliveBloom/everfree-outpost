#![crate_name = "uvedit_asm"]
#![no_std]
#![feature(core_intrinsics)]
#![allow(dead_code)]    // Much of Mesh is dead now, but will be needed 'soon'

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

#[macro_use] extern crate bitflags;
extern crate physics;

use std::cmp;
use std::collections::{BTreeMap, VecDeque};
use std::iter;
use std::slice;

use physics::v3::{V2, Vn, Region, scalar};


static mut STATE: *mut State = 0 as *mut _;

#[no_mangle]
pub unsafe extern fn init() {
    println!("before");
    STATE = Box::into_raw(Box::new(State::new()));
    println!("after");
}

#[no_mangle]
pub unsafe extern fn get_lines_ptr() -> *const (V2, V2) {
    (*STATE).lines.as_ptr()
}

#[no_mangle]
pub unsafe extern fn get_lines_len() -> usize {
    (*STATE).lines.len()
}

#[no_mangle]
pub unsafe extern fn get_overlay_ptr() -> *const (u8, u8, u8, u8) {
    (*STATE).overlay.as_ptr()
}

#[no_mangle]
pub unsafe extern fn get_mask_ptr() -> *mut u8 {
    (*STATE).mask.as_mut_ptr() as *mut u8
}

#[no_mangle]
pub unsafe extern fn load_sprite(ptr: *const (u8, u8, u8, u8),
                                 len: usize) {
    let slice = slice::from_raw_parts(ptr, len);
    (*STATE).load_sprite(slice)
}

#[no_mangle]
pub unsafe extern fn update() {
    (*STATE).update();
}

#[no_mangle]
pub unsafe extern fn handle_mouse_move(x: i32, y: i32) {
    (*STATE).handle_mouse_move(V2::new(x, y));
}

#[no_mangle]
pub unsafe extern fn handle_mouse_down(x: i32, y: i32, shift: u8) {
    (*STATE).handle_mouse_down(V2::new(x, y), shift != 0);
}

#[no_mangle]
pub unsafe extern fn handle_mouse_up(x: i32, y: i32) {
    (*STATE).handle_mouse_up(V2::new(x, y));
}

#[no_mangle]
pub unsafe extern fn set_mode(mode: usize) {
    (*STATE).set_mode(mode);
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Nothing,
    DragVertex(usize, V2),
    DrawMask(MaskBits),
    MaskOutline(MaskBits, V2),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    EditMesh,
    DrawMask(MaskBits),
}

bitflags! {
    flags MaskBits: u8 {
        const MASKED =      0x01,
        const NO_BORDER =   0x02,
    }
}

const OUTLINE_COLOR: u8 = 180;

pub struct State {
    mesh: Mesh,
    mask: Vec<MaskBits>,

    sprite: Vec<u8>,

    lines: Vec<(V2, V2)>,
    mouse_pos: V2,
    mode: Mode,
    action: Action,
    overlay: Vec<(u8, u8, u8, u8)>,
}

impl State {
    fn new() -> State {
        State {
            mesh: Mesh::new(),
            mask: iter::repeat(MASKED)
                      .take((SPRITE_SIZE * SPRITE_SIZE) as usize)
                      .collect(),
            sprite: iter::repeat(0)
                         .take((SPRITE_SIZE * SPRITE_SIZE) as usize)
                         .collect(),

            lines: Vec::new(),
            mouse_pos: scalar(-1),
            mode: Mode::DrawMask(MaskBits::empty()),
            action: Action::Nothing,
            overlay: iter::repeat((0, 0, 0, 0))
                         .take((SPRITE_SIZE * SPRITE_SIZE) as usize)
                         .collect(),
        }
    }

    fn set_mode(&mut self, mode: usize) {
        self.mode = match mode {
            0 => Mode::EditMesh,
            1 => Mode::DrawMask(MaskBits::empty()),
            2 => Mode::DrawMask(MASKED),
            3 => Mode::DrawMask(MASKED | NO_BORDER),
            _ => panic!("bad mode index")
        };
    }

    fn load_sprite(&mut self, sprite: &[(u8, u8, u8, u8)]) {
        assert!(sprite.len() == (SPRITE_SIZE * SPRITE_SIZE) as usize);
        for i in 0 .. sprite.len() {
            let (r,_g,_b,a) = sprite[i];
            self.sprite[i] = if a == 255 { r } else { 255 };
        }
    }

    fn update(&mut self) {
        self.lines.clear();

        if self.mode == Mode::EditMesh {
            self.mesh.draw(self.mouse_pos,
                           &mut self.lines);
        }

        for c in &mut self.overlay {
            *c = (0, 0, 0, 0);
        }

        let bounds = Region::<V2>::sized(scalar(SPRITE_SIZE));
        let mouse_px = self.mouse_pos.div_floor(scalar(RENDER_SCALE));

        for p in bounds.points() {
            let idx = bounds.index(p);
            if self.mask[idx].contains(NO_BORDER) {
                self.overlay[idx] = (0, 0, 0, 100);
            } else if self.mask[idx].contains(MASKED) {
                self.overlay[idx].2 = 255;
                self.overlay[idx].3 = 100;
            }
        }

        if let Action::MaskOutline(_, start_px) = self.action {
            for p in self.trace_outline(start_px, mouse_px) {
                self.overlay[bounds.index(p)] = (255, 0, 0, 100);
            }
        }

        if let Mode::DrawMask(_) = self.mode {
            if bounds.contains(mouse_px) {
                let idx = bounds.index(mouse_px);
                self.overlay[idx].1 = 255;
                self.overlay[idx].3 = 100;
            }
        }

        /*
        let overlay = &mut self.overlay;
        let mesh = &self.mesh;
        mesh.rasterize(|p, u_idx, v_idx| {
            if bounds.contains(p) {
                let uv = mesh.quad_uv(cell_center(p), u_idx, v_idx);

                let u = (FX_1 - uv.x) * (u_idx as i32) + (uv.x) * (u_idx as i32 + 1);
                let v = (FX_1 - uv.y) * (v_idx as i32) + (uv.y) * (v_idx as i32 + 1);

                let red = cmp::min(cmp::max(0, u / mesh.u_sub as i32), 255) as u8;
                let green = cmp::min(cmp::max(0, v / mesh.v_sub as i32), 255) as u8;

                overlay[bounds.index(p)] = (red, green, 255, 255);
            }
        });
        */
    }

    fn handle_mouse_move(&mut self, pos: V2) {
        self.mouse_pos = pos;

        let mouse_pos_fx = pixel_to_fixed(pos);
        match self.action {
            Action::Nothing => {},
            Action::DragVertex(v, off) => {
                let vert_pos = off + mouse_pos_fx;
                self.mesh.set_vertex_pos(v, vert_pos);
            },
            Action::DrawMask(val) => {
                let px_pos = pos.div_floor(scalar(RENDER_SCALE));
                let idx = Region::sized(scalar(SPRITE_SIZE)).index(px_pos);
                self.mask[idx] = val;
            },
            Action::MaskOutline(_, _) => {},
        }
    }

    fn handle_mouse_down(&mut self, pos: V2, shift: bool) {
        self.mouse_pos = pos;
        match self.mode {
            Mode::EditMesh => {
                if let Some(v) = self.mesh.hit_vertex(pos) {
                    let vert_pos = self.mesh.vertex_pos(v);
                    println!("hit {} @ {:?}", v, vert_pos);
                    let mouse_pos_fx = pixel_to_fixed(pos);
                    self.action = Action::DragVertex(v, vert_pos - mouse_pos_fx);
                }
            },

            Mode::DrawMask(val) => {
                let px_pos = pos.div_floor(scalar(RENDER_SCALE));
                if shift {
                    self.action = Action::MaskOutline(val, px_pos);
                } else {
                    self.action = Action::DrawMask(val);
                    self.handle_mouse_move(pos);
                }
            },
        }
    }

    fn handle_mouse_up(&mut self, pos: V2) {
        self.mouse_pos = pos;

        match self.action {
            Action::MaskOutline(val, start_px) => {
                let bounds = Region::sized(scalar(SPRITE_SIZE));
                let mouse_px = pos.div_floor(scalar(RENDER_SCALE));
                if mouse_px == start_px {
                    self.floodfill_mask(mouse_px, val);
                } else {
                    for p in self.trace_outline(start_px, mouse_px) {
                        self.mask[bounds.index(p)] = val;
                    }
                }
            },

            _ => {},
        }

        self.action = Action::Nothing;
    }


    fn floodfill_mask(&mut self, start: V2, val: MaskBits) {
        let bounds = Region::sized(scalar(SPRITE_SIZE));

        let mut queue = Vec::new();
        let old_val = self.mask[bounds.index(start)];
        if old_val == val {
            return;
            // (Otherwise we would loop forever, because we detect unvisited squares by
            // `mask[i] == old_val`.)
        }
        self.mask[bounds.index(start)] = val;
        queue.push(start);

        while let Some(cur) = queue.pop() {
            for &(dx, dy) in &[(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let next = cur + V2::new(dx, dy);
                if bounds.contains(next) && self.mask[bounds.index(next)] == old_val {
                    self.mask[bounds.index(next)] = val;
                    queue.push(next);
                }
            }
        }
    }

    fn trace_outline(&mut self, start: V2, end: V2) -> Vec<V2> {
        let bounds = Region::sized(scalar(SPRITE_SIZE));

        let mut queue = VecDeque::new();
        let mut parent = BTreeMap::new();
        queue.push_back((start, None));

        while let Some((cur, prev)) = queue.pop_front() {
            if parent.contains_key(&(cur.x, cur.y)) {
                continue;
            }
            parent.insert((cur.x, cur.y), prev);

            if cur == end {
                let mut trace = Vec::new();
                trace.push(end);
                loop {
                    let last = *trace.last().unwrap();
                    if let Some(prev) = parent[&(last.x, last.y)] {
                        trace.push(prev);
                    } else {
                        break;
                    }
                }
                trace.reverse();
                return trace;
            }

            for &(dx, dy) in &[(1, 0), (0, 1), (-1, 0), (0, -1),
                               (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let next = cur + V2::new(dx, dy);
                if bounds.contains(next) && self.sprite[bounds.index(next)] == OUTLINE_COLOR {
                    queue.push_back((next, Some(cur)));
                }
            }
        }

        Vec::new()
    }
}


const FX_BITS: usize = 8;
const FX_BASE: i32 = 1 << FX_BITS;
const FX_1: i32 = 1 << FX_BITS;
const RENDER_SCALE: i32 = 8;
const SPRITE_SIZE: i32 = 96;

struct Mesh {
    u_sub: u8,
    v_sub: u8,

    u_vals: Box<[i32]>,
    v_vals: Box<[i32]>,

    points: Box<[V2]>,
}

impl Mesh {
    fn new() -> Mesh {
        let a = SPRITE_SIZE * 1 / 4 << FX_BITS;
        let b = SPRITE_SIZE * 2 / 4 << FX_BITS;
        let c = SPRITE_SIZE * 3 / 4 << FX_BITS;
        let points = vec![V2::new(a, a), V2::new(b, a), V2::new(c, a),
                          V2::new(a, b), V2::new(b, b), V2::new(c, b),
                          V2::new(a, c), V2::new(b, c), V2::new(c, c)].into_boxed_slice();

        Mesh {
            u_sub: 2,
            v_sub: 2,

            u_vals: vec![0, 1 << FX_BITS].into_boxed_slice(),
            v_vals: vec![0, 1 << FX_BITS].into_boxed_slice(),

            points: points,
        }
    }

    fn bounds(&self) -> Region<V2> {
        Region::new(V2::new(0, 0),
                    V2::new(self.u_sub as i32 + 1, self.v_sub as i32 + 1))
    }

    fn vertex_pos(&self, v: usize) -> V2 {
        self.points[v]
    }

    fn set_vertex_pos(&mut self, v: usize, pos: V2) {
        self.points[v] = pos;
    }

    fn hit_vertex(&self, mouse_pos_px: V2) -> Option<usize> {
        let bounds = self.bounds();
        for u in 0 .. self.u_sub + 1 {
            for v in 0 .. self.v_sub + 1 {
                let pos = V2::new(u as i32, v as i32);
                let corner = self.points[bounds.index(pos)];
                let corner_px = fixed_to_pixel(corner);
                if (corner_px - mouse_pos_px).abs().max() <= 5 {
                    return Some(bounds.index(pos));
                }
            }
        }
        None
    }

    fn check_inside(&self, pos: V2) -> bool {
        let bounds = self.bounds();
        for u in 0 .. self.u_sub {
            for v in 0 .. self.v_sub {
                let cell = V2::new(u as i32, v as i32);
                let a = self.points[bounds.index(cell + V2::new(0, 0))];
                let b = self.points[bounds.index(cell + V2::new(1, 0))];
                let c = self.points[bounds.index(cell + V2::new(0, 1))];
                let d = self.points[bounds.index(cell + V2::new(1, 1))];

                let (b1, b2, b3) = calc_barycentric(a, b, c, pos);
                if b1 >= 0 && b2 >= 0 && b3 >= 0 {
                    return true;
                }

                let (b1, b2, b3) = calc_barycentric(b, c, d, pos);
                if b1 >= 0 && b2 >= 0 && b3 >= 0 {
                    return true;
                }
            }
        }
        false
    }

    fn check_inside_quad(&self, pos: V2, u: u8, v: u8) -> bool {
        let bounds = self.bounds();
                let cell = V2::new(u as i32, v as i32);
                let a = self.points[bounds.index(cell + V2::new(0, 0))];
                let b = self.points[bounds.index(cell + V2::new(1, 0))];
                let c = self.points[bounds.index(cell + V2::new(0, 1))];
                let d = self.points[bounds.index(cell + V2::new(1, 1))];

                let (b1, b2, b3) = calc_barycentric(a, b, c, pos);
                if b1 >= 0 && b2 >= 0 && b3 >= 0 {
                    return true;
                }

                let (b1, b2, b3) = calc_barycentric(b, c, d, pos);
                if b1 >= 0 && b2 >= 0 && b3 >= 0 {
                    return true;
                }
        false
    }

    fn quad_uv(&self, pos: V2, i: u8, j: u8) -> V2 {
        let bounds = self.bounds();
        let cell = V2::new(i as i32, j as i32);
        let a = self.points[bounds.index(cell + V2::new(0, 0))];
        let b = self.points[bounds.index(cell + V2::new(1, 0))];
        let c = self.points[bounds.index(cell + V2::new(1, 1))];
        let d = self.points[bounds.index(cell + V2::new(0, 1))];

        calc_quad_uv(a, b, c, d, pos)
    }

    fn rasterize<F>(&self, mut f: F)
            where F: FnMut(V2, u8, u8) {
        let bounds = self.bounds();
        for u in 0 .. self.u_sub {
            for v in 0 .. self.v_sub {
                let cell = V2::new(u as i32, v as i32);
                let a = self.points[bounds.index(cell + V2::new(0, 0))];
                let b = self.points[bounds.index(cell + V2::new(1, 0))];
                let c = self.points[bounds.index(cell + V2::new(0, 1))];
                let d = self.points[bounds.index(cell + V2::new(1, 1))];
                raster_tri(a, b, c, |p| f(p, u, v));
                raster_tri(b, c, d, |p| f(p, u, v));
            }
        }
    }

    fn draw(&self,
            mouse_pos_px: V2,
            lines: &mut Vec<(V2, V2)>) {
        let bounds = self.bounds();
        let mut hit_vertex = false;
        for u in 0 .. self.u_sub + 1 {
            for v in 0 .. self.v_sub + 1 {
                let pos = V2::new(u as i32, v as i32);
                let corner = self.points[bounds.index(pos)];

                if u < self.u_sub {
                    let right = self.points[bounds.index(pos + V2::new(1, 0))];
                    lines.push((fixed_to_pixel(corner),
                                fixed_to_pixel(right)));
                }

                if v < self.v_sub {
                    let below = self.points[bounds.index(pos + V2::new(0, 1))];
                    lines.push((fixed_to_pixel(corner),
                                fixed_to_pixel(below)));
                }

                let corner_px = fixed_to_pixel(corner);
                if !hit_vertex && (corner_px - mouse_pos_px).abs().max() <= 5 {
                    // Only mark the first vertex that's moused over.
                    hit_vertex = true;

                    let a = corner_px + V2::new(-5, -5);
                    let b = corner_px + V2::new( 5, -5);
                    let c = corner_px + V2::new( 5,  5);
                    let d = corner_px + V2::new(-5,  5);
                    lines.push((a, b));
                    lines.push((b, c));
                    lines.push((c, d));
                    lines.push((d, a));
                }
            }
        }
    }
}

fn fixed_to_pixel(p: V2) -> V2 {
    p * scalar(RENDER_SCALE) >> FX_BITS
}

fn pixel_to_fixed(p: V2) -> V2 {
    (p << FX_BITS) / scalar(RENDER_SCALE)
}

fn cell_center(p: V2) -> V2 {
    ((p << 1) + scalar(1)) << (FX_BITS - 1)
}

fn calc_barycentric(a: V2, b: V2, c: V2, p: V2) -> (i32, i32, i32) {
    let denom = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    let num1 = (b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y);
    let num2 = (c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y);

    let bary1 = (num1) / (denom >> FX_BITS);
    let bary2 = (num2) / (denom >> FX_BITS);
    let bary3 = (1 << FX_BITS) - (bary1 + bary2);
    (bary1, bary2, bary3)
}

fn flip(v: V2) -> V2 {
    V2::new(-v.y, v.x)
}

fn raster_tri<F>(a: V2, b: V2, c: V2, mut f: F)
        where F: FnMut(V2) {

    // Sort by y-coordinate
    let (a, b, c) = if b.y < a.y { (b, a, c) } else { (a, b, c) };
    let (a, b, c) = if c.y < b.y { (a, c, b) } else { (a, b, c) };
    let (a, b, c) = if b.y < a.y { (b, a, c) } else { (a, b, c) };

    let n_ab = flip(b - a);
    let n_ac = flip(c - a);
    let n_bc = flip(c - b);

    let bbox = Region::new(V2::new(cmp::min(cmp::min(a.x, b.x), c.x), a.y),
                           V2::new(cmp::max(cmp::max(a.x, b.x), c.x), c.y))
                   .div_round_signed(FX_BASE);
    for p in bbox.points() {
        let fx_pos = cell_center(p);
        if fx_pos.y < a.y || fx_pos.y > c.y {
            continue;
        }
        if fx_pos.y < b.y {
            let i = (fx_pos - a).dot(n_ab);
            let j = (fx_pos - a).dot(n_ac);
            if i.signum() * j.signum() <= 0 {
                f(p);
            }
        } else {
            let i = (fx_pos - b).dot(n_bc);
            let j = (fx_pos - a).dot(n_ac);
            if i.signum() * j.signum() <= 0 {
                f(p);
            }
        }
    }
}

/// Note that the vertices should be given in winding order, not grid order.
fn calc_quad_uv(a: V2, b: V2, c: V2, d: V2, x: V2) -> V2 {
    // Algorithm from http://www.iquilezles.org/www/articles/ibilinear/ibilinear.htm
    let e = b - a;
    let f = d - a;
    let g = a - b + c - d;
    let h = x - a;

    // 2*FX_BITS
    let k2 = g.cross(f);
    let k1 = e.cross(f) + h.cross(g);
    let k0 = h.cross(e);


    let v = if k2 != 0 {
        let root = sqrt((k1 as f64) * (k1 as f64) - 4.0 * (k0 as f64) * (k2 as f64)) as i32;

        let v1 = (-k1 + root) / ((2 * k2) >> FX_BITS);
        let v2 = (-k1 - root) / ((2 * k2) >> FX_BITS);

        if 0 <= v1 && v1 <= 1 << FX_BITS { v1 } else { v2 }
    } else {
        -k0 / (k1 >> FX_BITS)
    };

    let numer = h.x * FX_1 - f.x * v;
    let denom = e.x * FX_1 + g.x * v;
    let u = numer / (denom >> FX_BITS);

    V2::new(u, v)
}

fn sqrt(x: f64) -> f64 {
    unsafe { std::intrinsics::sqrtf64(x) }
}
