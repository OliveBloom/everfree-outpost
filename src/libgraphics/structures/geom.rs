use std::prelude::v1::*;
use std::ops::{Deref, DerefMut};
use std::ptr;

use physics::v3::{V3, V2, scalar, Region};
use physics::CHUNK_SIZE;

use types::{StructureTemplate, TemplatePart, TemplateVertex};

use super::Buffer;


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


pub struct TemplateData {
    pub templates: Box<[StructureTemplate]>,
    pub parts: Box<[TemplatePart]>,
    pub verts: Box<[TemplateVertex]>,
}

impl TemplateData {
    pub fn new(templates: Box<[StructureTemplate]>,
               parts: Box<[TemplatePart]>,
               verts: Box<[TemplateVertex]>) -> TemplateData {
        TemplateData {
            templates: templates,
            parts: parts,
            verts: verts,
        }
    }

    pub fn templates_ptr(&self) -> *mut StructureTemplate {
        self.templates.as_ptr() as *mut _
    }

    pub fn template_parts_ptr(&self) -> *mut TemplatePart {
        self.parts.as_ptr() as *mut _
    }

    pub fn template_verts_ptr(&self) -> *mut TemplateVertex {
        self.verts.as_ptr() as *mut _
    }
}

pub struct GeomGenState {
    bounds: Region<V2>,
    cur: usize,
    sheet: u8,
}

impl GeomGenState {
    pub fn new(bounds: Region<V2>, sheet: u8) -> GeomGenState {
        GeomGenState {
            bounds: bounds * scalar(CHUNK_SIZE),
            cur: 0,
            sheet: sheet,
        }
    }
}

pub struct GeomGen<'a> {
    buffer: &'a Buffer,
    data: &'a TemplateData,
    state: &'a mut GeomGenState,
}

impl<'a> GeomGen<'a> {
    pub fn new(buffer: &'a Buffer,
               data: &'a TemplateData,
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
        while *idx < buf.len() {
            if self.cur >= self.buffer.len() {
                // No more structures
                return false;
            }

            let s = &self.buffer[self.cur];
            self.cur += 1;

            let t = &self.data.templates[s.template_id as usize];

            let s_pos = V3::new(s.pos.0 as i32,
                                s.pos.1 as i32,
                                s.pos.2 as i32);
            if !self.bounds.contains(s_pos.reduce()) {
                // Not visible
                continue;
            }

            if *idx + t.vert_count as usize >= buf.len() {
                // Not enough space for all this structure's vertices.  Bailing out in this case
                // means we don't have to deal with tracking partially-emitted structures.  Though
                // we do have to adjust self.cur to avoid skipping `s` completely.
                self.cur -= 1;
                break;
            }

            let i0 = t.part_idx as usize;
            let i1 = i0 + t.part_count as usize;
            for p in &self.data.parts[i0 .. i1] {
                if p.sheet != self.sheet {
                    continue;
                }

                let j0 = p.vert_idx as usize;
                let j1 = j0 + p.vert_count as usize;
                for v in &self.data.verts[j0 .. j1] {
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

        true
    }
}

impl<'a> Deref for GeomGen<'a> {
    type Target = GeomGenState;
    fn deref(&self) -> &GeomGenState {
        &self.state
    }
}

impl<'a> DerefMut for GeomGen<'a> {
    fn deref_mut(&mut self) -> &mut GeomGenState {
        &mut self.state
    }
}
