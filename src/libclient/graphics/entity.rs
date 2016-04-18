use std::prelude::v1::*;

use physics::v3::{V3, V2, scalar, Region};
use physics::{CHUNK_SIZE, CHUNK_BITS, TILE_SIZE, TILE_BITS};

use data::Data;
use entity::Entities;
use platform::gl;
use terrain::LOCAL_BITS;
use util;

use graphics::GeometryGenerator;


#[derive(Clone, Copy)]
pub struct Vertex {
    // 0
    dest_pos: (u16, u16),
    src_pos: (u16, u16),
    sheet: u8,
    color: (u8, u8, u8),

    // 12
    ref_pos: (u16, u16, u16),
    ref_size_z: u16,

    // 20
    anim_length: u16,
    anim_rate: u16,
    anim_start: u16,
    anim_step: u16,

    // 28
}

pub fn load_shader<GL: gl::Context>(gl: &mut GL) -> GL::Shader {
    gl.load_shader(
        "entity2.vert", "entity2.frag", "",
        uniforms! {
            camera_pos: V2,
            camera_size: V2,
            now: Float,
        },
        arrays! {
            // struct 
            [28] attribs! {
                dest_pos: U16[2] @0,
                src_pos: U16[2] @4,
                sheet: U8[1] @8,
                color: U8[3] (norm) @9,
                // Combine ref_pos and ref_size_z to avoid exceeding 8 attribs
                ref_pos_size: U16[4] @12,
                // Combine all anim properties as well
                anim_info: U16[4] @20,
            },
        },
        textures! {
            sheet_tex,
            depth_tex,
        },
        outputs! { color: 1, depth })
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
            let num_layers = count_layers(e.appearance);
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

            let num_layers = count_layers(e.appearance);
            if idx + 6 * num_layers >= buf.len() {
                return (idx, true);
            }

            let a = &self.data.animations[e.motion.anim_id as usize];

            for_each_layer(e.appearance, |layer_table_idx, color| {
                let layer_idx = self.data.pony_layer_table[layer_table_idx];
                let l = &self.data.sprite_layers[layer_idx as usize];
                let g = &self.data.sprite_graphics[(l.gfx_start + a.local_id) as usize];

                // Top-left corner of the output rect
                let dest_x = (pos.x - 32) as u16;
                let dest_y = (pos.y - pos.z - 64) as u16;

                for &(cx, cy) in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
                    let dest_pos =
                        if g.mirror == 0 {
                            (dest_x + g.dest_offset.0 + cx * g.size.0,
                             dest_y + g.dest_offset.1 + cy * g.size.1)
                        } else {
                            // TODO: hardcoded sprite size
                            (dest_x + (96 - g.dest_offset.0 - g.size.0) + (1 - cx) * g.size.0,
                             dest_y + g.dest_offset.1 + cy * g.size.1)
                        };

                    buf[idx] = Vertex {
                        dest_pos: dest_pos,
                        src_pos: (g.src_offset.0 + cx * g.size.0,
                                  g.src_offset.1 + cy * g.size.1),
                        sheet: g.sheet,
                        color: color,

                        ref_pos: (pos.x as u16,
                                  // TODO: hardcoded size
                                  // TODO: extra hack - shouldn't need this +1
                                  pos.y as u16 + 1/*32*/,
                                  pos.z as u16),
                        // TODO: hardcoded size
                        ref_size_z: 64,

                        anim_length: a.length as u16,
                        anim_rate: a.framerate as u16,
                        anim_start: (e.motion.start_time % 55440) as u16,
                        anim_step: g.size.0,
                    };
                    idx += 1;
                }
            });
        }

        // Ran out of entites - we're done.
        (idx, false)
    }
}


const WINGS: u32 = 1 << 6;
const HORN: u32 = 1 << 7;
const STALLION: u32 = 1 << 8;
const MANE_SHIFT: usize = 10;
const TAIL_SHIFT: usize = 13;
const EQUIP0_SHIFT: usize = 18;
const EQUIP1_SHIFT: usize = 22;
const EQUIP2_SHIFT: usize = 26;

static COLOR_TABLE: [u8; 6] = [0x00, 0x44, 0x88, 0xcc, 0xff, 0xff];

fn count_layers(appearance: u32) -> usize {
    let mut count = 1;  // base
    if appearance & WINGS != 0 { count += 2; }  // frontwing + backwing
    if appearance & HORN != 0 { count += 1; }   // horn
    count += 3;     // mane + tail + eyes
    if (appearance >> EQUIP0_SHIFT) & 0xf != 0 { count += 1; }  // equip0
    if (appearance >> EQUIP1_SHIFT) & 0xf != 0 { count += 1; }  // equip1
    if (appearance >> EQUIP2_SHIFT) & 0xf != 0 { count += 1; }  // equip2
    count
}

fn for_each_layer<F: FnMut(usize, (u8, u8, u8))>(appearance: u32, mut f: F) {
    let red = (appearance as usize >> 4) & 3;
    let green = (appearance as usize >> 2) & 3;
    let blue = (appearance as usize >> 0) & 3;
    let wings = appearance & WINGS != 0;
    let horn = appearance & HORN != 0;
    let stallion = appearance & STALLION != 0;
    let mane = (appearance >> MANE_SHIFT) & 7;
    let tail = (appearance >> TAIL_SHIFT) & 7;
    let equip0 = (appearance >> EQUIP0_SHIFT) & 15;

    let tint0 = (COLOR_TABLE[red + 1],
                 COLOR_TABLE[green + 1],
                 COLOR_TABLE[blue + 1]);

    let tint1 = (COLOR_TABLE[red + 2],
                 COLOR_TABLE[green + 2],
                 COLOR_TABLE[blue + 2]);

    let white = (0xff, 0xff, 0xff);

    let mut go = |x, color| {
        f(x * 2 + stallion as usize, color)
    };

    if wings {
        go(3, tint1);
    }
    go(0, tint1); // base
    if wings {
        go(2, tint1);
    }
    go(4, white); // eyes
    go(5 + mane as usize, tint0);
    if horn {
        go(1, tint1);
    }
    go(8 + tail as usize, tint0);
    if equip0 != 0 {
        go(11 + equip0 as usize - 1, white);
    }
}
