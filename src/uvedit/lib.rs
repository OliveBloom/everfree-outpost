#![crate_name = "uvedit_asm"]
#![no_std]
#![feature(core_intrinsics)]

#[macro_use] extern crate fakestd as std;
use std::prelude::v1::*;

extern crate physics;

use std::cmp;
use std::iter;

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
pub unsafe extern fn update() {
    (*STATE).update();
}

#[no_mangle]
pub unsafe extern fn handle_mouse_move(x: i32, y: i32) {
    (*STATE).handle_mouse_move(V2::new(x, y));
}

#[no_mangle]
pub unsafe extern fn handle_mouse_down(x: i32, y: i32) {
    (*STATE).handle_mouse_down(V2::new(x, y));
}

#[no_mangle]
pub unsafe extern fn handle_mouse_up(x: i32, y: i32) {
    (*STATE).handle_mouse_up(V2::new(x, y));
}


enum Action {
    Nothing,
    DragVertex(usize, V2),
}

pub struct State {
    mesh: Mesh,
    lines: Vec<(V2, V2)>,
    mouse_pos: V2,
    action: Action,
    overlay: Vec<(u8, u8, u8, u8)>,
}

impl State {
    fn new() -> State {
        State {
            mesh: Mesh::new(),
            lines: Vec::new(),
            mouse_pos: scalar(-1),
            action: Action::Nothing,
            overlay: iter::repeat((0, 0, 0, 0))
                         .take((SPRITE_SIZE * SPRITE_SIZE) as usize)
                         .collect(),
        }
    }

    fn update(&mut self) {
        self.lines.clear();
        self.mesh.draw(self.mouse_pos,
                       &mut self.lines);

        for c in &mut self.overlay {
            *c = (0, 0, 0, 0);
        }

        let bounds = Region::sized(scalar(SPRITE_SIZE));

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
        }
    }

    fn handle_mouse_down(&mut self, pos: V2) {
        self.mouse_pos = pos;
        if let Some(v) = self.mesh.hit_vertex(pos) {
            let vert_pos = self.mesh.vertex_pos(v);
            println!("hit {} @ {:?}", v, vert_pos);
            let mouse_pos_fx = pixel_to_fixed(pos);
            self.action = Action::DragVertex(v, vert_pos - mouse_pos_fx);
        }
    }

    fn handle_mouse_up(&mut self, pos: V2) {
        self.mouse_pos = pos;
        self.action = Action::Nothing;
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
        let low = SPRITE_SIZE * 1 / 4 << FX_BITS;
        let high = SPRITE_SIZE * 3 / 4 << FX_BITS;
        let points = vec![V2::new(low, low),     V2::new(high, low),
                          V2::new(low, high),    V2::new(high, high)].into_boxed_slice();

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
        let mut hit_vertex = false;
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

    let numer = (h.x * FX_1 - f.x * v);
    let denom = (e.x * FX_1 + g.x * v);
    let u = numer / (denom >> FX_BITS);

    V2::new(u, v)
}

fn sqrt(x: f64) -> f64 {
    unsafe { std::intrinsics::sqrtf64(x) }
}
