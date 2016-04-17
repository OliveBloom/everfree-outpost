use std::prelude::v1::*;

use physics::v3::{V3, V2, scalar, Region};
use physics::{CHUNK_SIZE, CHUNK_BITS, TILE_SIZE, TILE_BITS};

use data::Data;
use entity::Entities;
use terrain::LOCAL_BITS;
use util;

use graphics::GeometryGenerator;


#[derive(Clone, Copy)]
pub struct Vertex {
    // 0
    dest_pos: (u16, u16),
    src_pos: (u16, u16),
    sheet: u8,
    _pad0: u8,

    // 10
    ref_pos: (u16, u16, u16),
    ref_size_z: u16,

    // 18
    anim_length: i8,
    anim_rate: u8,
    anim_start: u16,
    anim_step: u16,

    // 24
}


pub struct GeomGen<'a> {
    entities: &'a Entities,
    data: &'a Data,
    bounds: Region<V2>,
    now: i32,
    next: u32,
}

const LOCAL_PX_MASK: i32 = (1 << (TILE_BITS + CHUNK_BITS + LOCAL_BITS)) - 1;

impl<'a> GeomGen<'a> {
    pub fn new(entities: &'a Entities,
               data: &'a Data,
               chunk_bounds: Region<V2>,
               now: i32) -> GeomGen<'a> {
        let bounds = chunk_bounds * scalar(CHUNK_SIZE * TILE_SIZE);
        let bounds = Region::new(bounds.min - scalar(128),
                                 bounds.max);

        GeomGen {
            entities: entities,
            data: data,
            bounds: bounds,
            now: now,
            next: 0,
        }
    }

    pub fn count_verts(&self) -> usize {
        let mut count = 0;
        for (_, e) in self.entities.iter() {
            let pos = e.pos(self.now);
            if !util::contains_wrapped(self.bounds, pos.reduce(), scalar(LOCAL_PX_MASK)) {
                // Not visible
                continue;
            }

            //let t = &self.data.templates[s.template_id as usize];
            //count += t.vert_count as usize;
            let num_layers = 1;
            count += 6 * num_layers;
        }
        count
    }
}

impl<'a> GeometryGenerator for GeomGen<'a> {
    type Vertex = Vertex;

    fn generate(&mut self,
                buf: &mut [Vertex]) -> (usize, bool) {
        let mut idx = 0;
        for (&id, e) in self.entities.iter_from(self.next) {
            self.next = id;

            let pos = e.pos(self.now);
            if !util::contains_wrapped(self.bounds, pos.reduce(), scalar(LOCAL_PX_MASK)) {
                // Not visible
                continue;
            }

            let num_layers = 1;
            if idx + 6 * num_layers >= buf.len() {
                return (idx, true);
            }

            let a = &self.data.animations[e.motion.anim_id as usize];
            let l = &self.data.sprite_layers[1];
            let g = &self.data.sprite_graphics[(l.gfx_start + a.local_id) as usize];

            // Top-left corner of the output rect
            let dest_x = (pos.x - 48) as u16;
            let dest_y = (pos.y - pos.z - 80) as u16;

            for &(cx, cy) in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
                buf[idx] = Vertex {
                    dest_pos: (dest_x + g.dest_offset.0 + cx * g.size.0,
                               dest_y + g.dest_offset.1 + cy * g.size.1),
                    src_pos: (g.src_offset.0 + cx * g.size.0,
                              g.src_offset.1 + cy * g.size.1),
                    sheet: g.sheet,
                    _pad0: 0,

                    ref_pos: (pos.x as u16,
                              pos.y as u16,
                              pos.z as u16),
                    ref_size_z: 64,

                    anim_length: a.length as i8,
                    anim_rate: a.framerate,
                    anim_start: (e.motion.start_time % 55440) as u16,
                    anim_step: g.size.0,
                };
                idx += 1;
            }
        }

        // Ran out of entites - we're done.
        (idx, false)
    }
}
