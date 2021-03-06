use std::prelude::v1::*;

use physics::v3::{V3, V2, scalar, Region};
use physics::CHUNK_SIZE;

use data::Data;
use platform::gl;
use structures::Structures;
use terrain::LOCAL_MASK;
use util;

use graphics::GeometryGenerator;


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

pub fn load_shader<GL: gl::Context>(gl: &mut GL, shadow: bool) -> GL::Shader {
    gl.load_shader(
        "structure2.vert", "structure2.frag",
        if !shadow {
            defs! {
                SLICE_ENABLE: "1",
                SLICE_SIMPLIFIED: "1",
            }
        } else {
            defs! {
                SLICE_ENABLE: "1",
                SLICE_SIMPLIFIED: "1",
                OUTPOST_SHADOW: "1",
            }
        },
        uniforms! {
            cameraPos: V2,
            cameraSize: V2,
            sliceCenter: V2,
            sliceZ: Float,
            now: Float,
        },
        arrays! {
            // struct structure::Vertex
            [20] attribs! {
                vertOffset: U16[3] @0,
                animLength: I8[1] @6,
                animRate: U8[1] @7,
                blockPos: U8[3] @8,
                layer: U8[1] @11,
                displayOffset: I16[2] @12,
                animOneshotStart: U16[1] @16,
                animStep: U16[1] @18,
            },
        },
        textures! {
            sheetTex,
            cavernTex,
        },
        outputs! { color: 2, depth })
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

            let t = self.data.template(s.template_id);
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

            let t = self.data.template(s.template_id);

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
            for p in &self.data.template_parts()[i0 .. i1] {
                let j0 = p.vert_idx as usize;
                let j1 = j0 + p.vert_count as usize;
                for v in &self.data.template_verts()[j0 .. j1] {
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
