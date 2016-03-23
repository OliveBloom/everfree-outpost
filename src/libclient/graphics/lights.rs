use std::prelude::v1::*;
use std::ops::{Deref, DerefMut};
use std::ptr;

use physics::v3::{V3, V2, scalar, Region};
use physics::{CHUNK_BITS, CHUNK_SIZE, TILE_BITS, TILE_SIZE};

use structures::Structures;

use graphics::IntrusiveCorner;
use graphics::{emit_quad, remaining_quads};
use graphics::LOCAL_BITS;
use graphics::types::{StructureTemplate, HAS_LIGHT};



#[derive(Clone, Copy)]
pub struct Vertex {
    // 0
    corner: (u8, u8),
    center: (u16, u16, u16),

    // 8
    color: (u8, u8, u8),
    _pad1: u8,
    radius: u16,
    _pad2: u16,

    // 16
}

impl IntrusiveCorner for Vertex {
    fn corner(&self) -> &(u8, u8) { &self.corner }
    fn corner_mut(&mut self) -> &mut (u8, u8) { &mut self.corner }
}

pub struct GeomGenState {
    bounds: Region<V2>,
    next: u32,
}

impl GeomGenState {
    pub fn new(bounds: Region<V2>) -> GeomGenState {
        GeomGenState {
            bounds: bounds * scalar(CHUNK_SIZE * TILE_SIZE),
            next: 0,
        }
    }
}

pub struct GeomGen<'a> {
    buffer: &'a Structures,
    templates: &'a [StructureTemplate],
    state: &'a mut GeomGenState,
}

impl<'a> GeomGen<'a> {
    pub fn new(buffer: &'a Structures,
               templates: &'a [StructureTemplate],
               state: &'a mut GeomGenState) -> GeomGen<'a> {
        GeomGen {
            buffer: buffer,
            templates: templates,
            state: state,
        }
    }


    pub fn generate(&mut self,
                    buf: &mut [Vertex],
                    idx: &mut usize) -> bool {
        for (&id, s) in self.buffer.iter_from(self.state.next) {
            self.state.next = id;

            let t = &self.templates[s.template_id as usize];

            if !t.flags.contains(HAS_LIGHT) {
                continue;
            }

            // Be careful to avoid emitting duplicate geometry.  Two copies of a structure looks
            // the same as one, but two copies of a light is twice as bright.
            let offset = V3::new(t.light_pos.0 as i32,
                                 t.light_pos.1 as i32,
                                 t.light_pos.2 as i32);
            let s_pos = V3::new(s.pos.0 as i32,
                                s.pos.1 as i32,
                                s.pos.2 as i32);
            let center = s_pos * scalar(TILE_SIZE) + offset;

            // Do a wrapped version of `self.bounds.contains(center)`.
            const MASK: i32 = (1 << (LOCAL_BITS + CHUNK_BITS + TILE_BITS)) - 1;
            let wrapped_center = (center.reduce() - self.state.bounds.min) & scalar(MASK);
            let wrapped_bounds = self.state.bounds - self.state.bounds.min;
            if !wrapped_bounds.contains(wrapped_center) {
                continue;
            }

            emit_quad(buf, idx, Vertex {
                corner: (0, 0),
                // Give the position of the front corner of the structure, since the quad should
                // cover the front plane.
                center: (center.x as u16,
                         center.y as u16,
                         center.z as u16),

                color: t.light_color,
                radius: t.light_radius,

                _pad1: 0,
                _pad2: 0,
            });
        }

        // Ran out of structures - we're done.
        false
    }
}