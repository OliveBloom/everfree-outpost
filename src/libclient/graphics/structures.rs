use std::prelude::v1::*;
use std::ops::{Deref, DerefMut};
use std::ptr;

use physics::v3::{V3, V2, scalar, Region};
use physics::CHUNK_SIZE;

use data::Data;
use structures::Structures;

use graphics::types::{StructureTemplate, TemplatePart, TemplateVertex};


#[derive(Clone, Copy)]
pub struct Vertex {
    // 0
    vert_offset: (u16, u16, u16),
    anim_length: i8,
    anim_rate: u8,

    // 8
    struct_pos: (u8, u8, u8),
    layer: u8,
    display_offset: (i16, i16),

    // 16
    anim_oneshot_start: u16,
    anim_step: u16,

    // 20
}


pub struct GeomGenState {
    bounds: Region<V2>,
    next: u32,
    sheet: u8,
}

impl GeomGenState {
    pub fn new(bounds: Region<V2>, sheet: u8) -> GeomGenState {
        GeomGenState {
            bounds: bounds * scalar(CHUNK_SIZE),
            next: 0,
            sheet: sheet,
        }
    }
}

pub struct GeomGen<'a> {
    buffer: &'a Structures,
    data: &'a Data,
    state: &'a mut GeomGenState,
}

impl<'a> GeomGen<'a> {
    pub fn new(buffer: &'a Structures,
               data: &'a Data,
               state: &'a mut GeomGenState) -> GeomGen<'a> {
        GeomGen {
            buffer: buffer,
            data: data,
            state: state,
        }
    }

    pub fn generate(&mut self,
                    buf: &mut [Vertex],
                    idx: &mut usize) -> bool {
        for (&id, s) in self.buffer.iter_from(self.state.next) {
            self.state.next = id;

            let t = &self.data.templates[s.template_id as usize];

            let s_pos = V3::new(s.pos.0 as i32,
                                s.pos.1 as i32,
                                s.pos.2 as i32);
            if !self.state.bounds.contains(s_pos.reduce()) {
                // Not visible
                continue;
            }

            if *idx + t.vert_count as usize >= buf.len() {
                // Not enough space for all this structure's vertices.  Bailing out in this case
                // means we don't have to deal with tracking partially-emitted structures.  On the
                // next call, we'll start at `self.state.next`, which was already set to the
                // current structure's `id`.
                return true;
            }

            let i0 = t.part_idx as usize;
            let i1 = i0 + t.part_count as usize;
            for p in &self.data.template_parts[i0 .. i1] {
                if p.sheet != self.state.sheet {
                    continue;
                }

                let j0 = p.vert_idx as usize;
                let j1 = j0 + p.vert_count as usize;
                for v in &self.data.template_verts[j0 .. j1] {
                    buf[*idx] = Vertex {
                        vert_offset: (v.x, v.y, v.z),
                        anim_length: p.anim_length,
                        anim_rate: p.anim_rate,
                        struct_pos: s.pos,
                        layer: t.layer,
                        display_offset: p.offset,
                        anim_oneshot_start: s.oneshot_start,
                        anim_step: p.anim_step,
                    };
                    *idx += 1;
                }
            }
        }

        // Ran out of structures - we're done.
        false
    }
}
