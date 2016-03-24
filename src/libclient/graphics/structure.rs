use std::prelude::v1::*;
use std::ops::{Deref, DerefMut};
use std::ptr;

use physics::v3::{V3, V2, scalar, Region};
use physics::CHUNK_SIZE;

use data::Data;
use structures::Structures;
use terrain::LOCAL_MASK;
use util;

use graphics::GeometryGenerator;
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


pub struct GeomGen<'a> {
    structures: &'a Structures,
    data: &'a Data,
    bounds: Region<V2>,
    next: u32,
}

impl<'a> GeomGen<'a> {
    pub fn new(structures: &'a Structures,
               data: &'a Data,
               bounds: Region<V2>) -> GeomGen<'a> {
        GeomGen {
            structures: structures,
            data: data,
            bounds: bounds * scalar(CHUNK_SIZE),
            next: 0,
        }
    }

    pub fn count_verts(&self) -> usize {
        let mut count = 0;
        for (_, s) in self.structures.iter() {
            let s_pos = V3::new(s.pos.0 as i32,
                                s.pos.1 as i32,
                                s.pos.2 as i32);
            if !util::contains_wrapped(self.bounds, s_pos.reduce(), scalar(LOCAL_MASK)) {
                // Not visible
                continue;
            }

            let t = &self.data.templates[s.template_id as usize];
            count += t.vert_count as usize;
        }
        count
    }
}

impl<'a> GeometryGenerator for GeomGen<'a> {
    type Vertex = Vertex;

    fn generate(&mut self,
                buf: &mut [Vertex]) -> (usize, bool) {
        let mut idx = 0;
        for (&id, s) in self.structures.iter_from(self.next) {
            self.next = id;

            let t = &self.data.templates[s.template_id as usize];

            let s_pos = V3::new(s.pos.0 as i32,
                                s.pos.1 as i32,
                                s.pos.2 as i32);
            if !util::contains_wrapped(self.bounds, s_pos.reduce(), scalar(LOCAL_MASK)) {
                // Not visible
                continue;
            }

            if idx + t.vert_count as usize >= buf.len() {
                // Not enough space for all this structure's vertices.  Bailing out in this case
                // means we don't have to deal with tracking partially-emitted structures.  On the
                // next call, we'll start at `self.state.next`, which was already set to the
                // current structure's `id`.
                return (idx, true);
            }

            let i0 = t.part_idx as usize;
            let i1 = i0 + t.part_count as usize;
            for p in &self.data.template_parts[i0 .. i1] {
                let j0 = p.vert_idx as usize;
                let j1 = j0 + p.vert_count as usize;
                for v in &self.data.template_verts[j0 .. j1] {
                    buf[idx] = Vertex {
                        vert_offset: (v.x, v.y, v.z),
                        anim_length: p.anim_length,
                        anim_rate: p.anim_rate,
                        struct_pos: s.pos,
                        layer: t.layer,
                        display_offset: p.offset,
                        anim_oneshot_start: s.oneshot_start,
                        anim_step: p.anim_step,
                    };
                    idx += 1;
                }
            }
        }

        // Ran out of structures - we're done.
        (idx, false)
    }
}
