#[macro_use] extern crate bitflags;
extern crate libc;
extern crate physics;
extern crate python3_sys;

mod python;
pub use python::PyInit_equip_sprites_render;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::iter;
use std::slice;

use physics::v3::{V2, Vn, Region, scalar};


fn make_array<T: Copy>(x: T, len: usize) -> Box<[T]> {
    iter::repeat(x).take(len).collect::<Vec<T>>().into_boxed_slice()
}

fn color_slice<'a>(raw: &'a [u8]) -> &'a [(u8, u8, u8, u8)] {
    unsafe {
        slice::from_raw_parts(raw.as_ptr() as *const (u8, u8, u8, u8),
                              raw.len() / 4)
    }
}

fn byte_slice<'a>(color: &'a [(u8, u8, u8, u8)]) -> &'a [u8] {
    unsafe {
        slice::from_raw_parts(color.as_ptr() as *const u8,
                              color.len() * 4)
    }
}

fn mask_bits_slice<'a>(raw: &'a [u8]) -> &'a [MaskBits] {
    unsafe {
        slice::from_raw_parts(raw.as_ptr() as *const MaskBits,
                              raw.len())
    }
}

bitflags! {
    flags MaskBits: u8 {
        const MASKED =      0x01,
        const NO_BORDER =   0x02,
    }
}


pub enum Style {
    Solid(u8, u8, u8),
}


pub struct Renderer {
    size: V2,
    style: Style,
    base: Box<[(u8, u8, u8, u8)]>,
    equip: Box<[(u8, u8, u8, u8)]>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            size: scalar(0),
            style: Style::Solid(255, 255, 255),
            base: make_array((0, 0, 0, 0), 0),
            equip: make_array((0, 0, 0, 0), 0),
        }
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn set_base(&mut self, size: V2, slice: &[u8]) {
        let px_count = size.x as usize * size.y as usize;
        assert!(px_count * 4 == slice.len());
        let slice = color_slice(slice);

        self.size = size;
        self.base = slice.to_owned().into_boxed_slice();
        self.equip = make_array((0, 0, 0, 0), px_count);
    }

    pub fn get_image(&self) -> &[u8] {
        byte_slice(&*self.equip)
    }

    pub fn render_part(&mut self, mask: &[u8]) {
        let px_count = self.size.x as usize * self.size.y as usize;
        assert!(px_count == mask.len());
        let mask = mask_bits_slice(mask);

        let bounds = Region::sized(self.size);
        for p in bounds.points() {
            let idx = bounds.index(p);
            if mask[idx].contains(MASKED) {
                continue;
            }

            let mut is_border = false;
            for &(dx, dy) in &[(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let q = p + V2::new(dx, dy);
                if !bounds.contains(q) {
                    continue;
                }
                let bits = mask[bounds.index(q)];
                if bits.contains(MASKED) && !bits.contains(NO_BORDER) {
                    is_border = true;
                    break;
                }
            }
            let is_border = is_border;

            let color = self.render_pixel(p, is_border);
            self.equip[idx] = color;
        }
    }

    fn render_pixel(&self, pos: V2, is_border: bool) -> (u8, u8, u8, u8) {
        let (r, g, b) = match self.style {
            Style::Solid(r, g, b) => (r, g, b),
        };

        let bounds = Region::sized(self.size);
        let c = (if is_border { 180 } else { self.base[bounds.index(pos)].0 }) as u16;

        ((r as u16 * c / 255) as u8,
         (g as u16 * c / 255) as u8,
         (b as u16 * c / 255) as u8,
         255)
    }
}
