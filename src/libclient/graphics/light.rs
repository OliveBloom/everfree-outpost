//use std::prelude::v1::*;
use types::*;
use physics::v3::{V3, V2, scalar, Region};
use physics::{CHUNK_BITS, CHUNK_SIZE, TILE_BITS, TILE_SIZE};

use entity::Entities;
use platform::gl;
use structures::Structures;
use util;

use graphics::{IntrusiveCorner, GeometryGenerator};
use graphics::{emit_quad, remaining_quads};
use graphics::entity;
use graphics::types::{StructureTemplate, HAS_LIGHT};
use terrain::LOCAL_BITS;



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

pub fn load_shader<GL: gl::Context>(gl: &mut GL) -> GL::Shader {
    gl.load_shader(
        "light2.vert", "light2.frag",
        defs! {
            LIGHT_INPUT: "attribute",
        },
        uniforms! {
            cameraPos: V2,
            cameraSize: V2,
        },
        arrays! {
            // struct 
            [16] attribs! {
                corner: U8[2] @0,
                center: U16[3] @2,
                colorIn: U8[3] (norm) @8,
                radiusIn: U16[1] @12,
            },
        },
        textures! {
            depthTex,
            entityDepthTex,
        },
        outputs! { color: 1 })
}


pub struct StructureGeomGen<'a> {
    structures: &'a Structures,
    templates: &'a [StructureTemplate],
    bounds: Region<V2>,
    next: StructureId,
}

impl<'a> StructureGeomGen<'a> {
    pub fn new(structures: &'a Structures,
               templates: &'a [StructureTemplate],
               bounds: Region<V2>) -> StructureGeomGen<'a> {
        StructureGeomGen {
            structures: structures,
            templates: templates,
            bounds: bounds * scalar(CHUNK_SIZE * TILE_SIZE),
            next: StructureId(0),
        }
    }

    pub fn count_verts(&self) -> usize {
        let mut count = 0;
        for (_, s) in self.structures.iter() {
            let t = &self.templates[s.template_id as usize];
            if !t.flags.contains(HAS_LIGHT) {
                continue;
            }

            let offset = V3::new(t.light_pos.0 as i32,
                                 t.light_pos.1 as i32,
                                 t.light_pos.2 as i32);
            let s_pos = V3::new(s.pos.0 as i32,
                                s.pos.1 as i32,
                                s.pos.2 as i32);
            let center = s_pos * scalar(TILE_SIZE) + offset;

            const MASK: i32 = (1 << (LOCAL_BITS + CHUNK_BITS + TILE_BITS)) - 1;
            if !util::contains_wrapped(self.bounds, center.reduce(), scalar(MASK)) {
                continue;
            }

            count += 6;
        }
        count
    }
}

impl<'a> GeometryGenerator for StructureGeomGen<'a> {
    type Vertex = Vertex;

    fn generate(&mut self, buf: &mut [Vertex]) -> (usize, bool) {
        let mut idx = 0;
        for (&id, s) in self.structures.iter_from(self.next) {
            self.next = id;

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

            const MASK: i32 = (1 << (LOCAL_BITS + CHUNK_BITS + TILE_BITS)) - 1;
            if !util::contains_wrapped(self.bounds, center.reduce(), scalar(MASK)) {
                continue;
            }

            if remaining_quads(buf, idx) < 1 {
                // No more space in buffer.
                return (idx, true);
            }

            emit_quad(buf, &mut idx, Vertex {
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
        (idx, false)
    }
}


pub struct EntityGeomGen<'a> {
    entities: &'a Entities,
    bounds: Region<V2>,
    now: Time,
    next: EntityId,
}

impl<'a> EntityGeomGen<'a> {
    pub fn new(entities: &'a Entities,
               bounds: Region<V2>,
               now: Time) -> EntityGeomGen<'a> {
        EntityGeomGen {
            entities: entities,
            bounds: bounds * scalar(CHUNK_SIZE * TILE_SIZE),
            now: now,
            next: EntityId(0),
        }
    }

    pub fn count_verts(&self) -> usize {
        let mut count = 0;
        for (_, e) in self.entities.iter() {
            if e.appearance & entity::LIGHT == 0 {
                continue;
            }

            let pos = e.pos(self.now);
            // TODO: hard-coded constant based on entity size
            let center = pos + V3::new(16, 16, 48);

            const MASK: i32 = (1 << (LOCAL_BITS + CHUNK_BITS + TILE_BITS)) - 1;
            if !util::contains_wrapped(self.bounds, center.reduce(), scalar(MASK)) {
                continue;
            }

            count += 6;
        }
        count
    }
}

impl<'a> GeometryGenerator for EntityGeomGen<'a> {
    type Vertex = Vertex;

    fn generate(&mut self, buf: &mut [Vertex]) -> (usize, bool) {
        let mut idx = 0;
        for (&id, e) in self.entities.iter_from(self.next) {
            self.next = id;

            if e.appearance & entity::LIGHT == 0 {
                continue;
            }

            let pos = e.pos(self.now);
            // TODO: hard-coded constant based on entity size
            let center = pos + V3::new(16, 16, 48);
            let center = util::wrap_base(center, self.bounds.min.extend(0));
            if !self.bounds.contains(center.reduce()) {
                // Not visible
                continue;
            }

            const MASK: i32 = (1 << (LOCAL_BITS + CHUNK_BITS + TILE_BITS)) - 1;
            if !util::contains_wrapped(self.bounds, center.reduce(), scalar(MASK)) {
                continue;
            }

            if remaining_quads(buf, idx) < 1 {
                // No more space in buffer.
                return (idx, true);
            }

            emit_quad(buf, &mut idx, Vertex {
                corner: (0, 0),
                // Give the position of the front corner of the entity, since the quad should
                // cover the front plane.
                center: (center.x as u16,
                         center.y as u16,
                         center.z as u16),

                color: (100, 180, 255),
                radius: 200,

                _pad1: 0,
                _pad2: 0,
            });
        }

        // Ran out of entities - we're done.
        (idx, false)
    }
}
