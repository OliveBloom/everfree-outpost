use std::prelude::v1::*;
use types::*;
use physics::v3::{V3, V2, scalar, Region};
use physics::{CHUNK_SIZE, CHUNK_BITS, TILE_SIZE, TILE_BITS};

use data::Data;
use entity::{Entities, Entity};
use fonts::{self, FontMetricsExt};
use platform::gl;
use predict::Predictor;
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
        "entity2.vert", "entity2.frag",
        defs! {
            SLICE_ENABLE: "1",
            SLICE_SIMPLIFIED: "1",
        },
        uniforms! {
            camera_pos: V2,
            camera_size: V2,
            // TODO: rename
            sliceCenter: V2,
            sliceZ: Float,
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
            cavernTex,
        },
        outputs! { color: 1, depth })
}


pub struct GeomGen<'a> {
    entities: &'a Entities,
    predictor: &'a Predictor,
    data: &'a Data,
    render_names: bool,
    bounds: Region<V2>,
    now: i32,
    future: i32,
    pawn_id: Option<EntityId>,
    next: Option<EntityId>,
}

const LOCAL_PX_MASK: i32 = (1 << (TILE_BITS + CHUNK_BITS + LOCAL_BITS)) - 1;

impl<'a> GeomGen<'a> {
    pub fn new(entities: &'a Entities,
               predictor: &'a Predictor,
               data: &'a Data,
               render_names: bool,
               chunk_bounds: Region<V2>,
               now: i32,
               future: i32,
               pawn_id: Option<EntityId>) -> GeomGen<'a> {
        let bounds = chunk_bounds * scalar(CHUNK_SIZE * TILE_SIZE);
        let bounds = Region::new(bounds.min - scalar(128),
                                 bounds.max);

        GeomGen {
            entities: entities,
            predictor: predictor,
            data: data,
            render_names: render_names,

            bounds: bounds,
            now: now,
            future: future,
            pawn_id: pawn_id,
            next: None,
        }
    }

    fn entity_pos(&self, id: EntityId, e: &Entity) -> V3 {
        if Some(id) != self.pawn_id {
            e.pos(self.now)
        } else {
            self.predictor.motion().pos(self.future)
        }
    }

    pub fn count_verts(&self) -> usize {
        let mut count = 0;
        for (&id, e) in self.entities.iter() {
            let pos = self.entity_pos(id, e);
            if !util::contains_wrapped(self.bounds, pos.reduce(), scalar(LOCAL_PX_MASK)) {
                // Not visible
                continue;
            }

            let is_pawn = Some(id) == self.pawn_id;
            count += 6 * count_quads_one(e, is_pawn);
        }
        count
    }
}

impl<'a> GeometryGenerator for GeomGen<'a> {
    type Vertex = Vertex;

    fn generate(&mut self,
                buf: &mut [Vertex]) -> (usize, bool) {
        let mut idx = 0;
        let iter = if let Some(next) = self.next {
            self.entities.iter_z_order_from(next)
        } else {
            self.entities.iter_z_order()
        };
        for e in iter {
            let id = e.id;
            self.next = Some(id);
            let is_pawn = Some(id) == self.pawn_id;

            let pos = self.entity_pos(id, e);
            let pos = util::wrap_base(pos, self.bounds.min.extend(0));
            if !self.bounds.contains(pos.reduce()) {
                // Not visible
                continue;
            }

            let num_quads = count_quads_one(e, is_pawn);
            if idx + 6 * num_quads >= buf.len() {
                return (idx, true);
            }

            let anim_id =
                if !is_pawn { e.motion.anim_id }
                else { self.predictor.motion().anim_id };
            let a = self.data.animation(anim_id);

            // Top-left corner of the output rect
            let dest_x = (pos.x - 32) as u16;
            let dest_y = (pos.y - pos.z - 64) as u16;

            const HACKY_ADJUSTMENT: u16 = 24;

            for_each_layer(e.appearance, |layer_table_idx, color| {
                let layer_idx = self.data.pony_layer_table()[layer_table_idx];
                let l = self.data.sprite_layer(layer_idx);
                let g = self.data.sprite_graphics_item(l.gfx_start + a.local_id);

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
                                  // TODO: arbitrary adjustment
                                  pos.y as u16 + HACKY_ADJUSTMENT,
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

            let should_render_name = self.render_names && !is_pawn;
            if let (true, &Some(ref name)) = (should_render_name, &e.name) {
                let name_center_x = dest_x + 48;
                let name_y = dest_y + 12;
                let name_x = name_center_x - fonts::NAME.measure_width(name) as u16 / 2;
                for (char_idx, offset) in fonts::NAME.iter_str(name) {
                    if let Some(char_idx) = char_idx {
                        let dx = name_x + offset as u16;
                        let dy = name_y;
                        let sx = fonts::NAME.xs[char_idx] as u16;
                        let sy = 0;
                        let w = fonts::NAME.widths[char_idx] as u16;
                        let h = fonts::NAME.height as u16;

                        for &(cx, cy) in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
                            buf[idx] = Vertex {
                                dest_pos: (dx + cx * w,
                                           dy + cy * h),
                                src_pos: (sx + cx * w,
                                          sy + cy * h),
                                sheet: 0,
                                color: (255, 255, 255),

                                ref_pos: (pos.x as u16,
                                          // TODO: hardcoded size
                                          // TODO: arbitrary adjustment
                                          pos.y as u16 + HACKY_ADJUSTMENT,
                                          pos.z as u16),
                                // TODO: hardcoded size
                                ref_size_z: 64,

                                anim_length: 1,
                                anim_rate: 1,
                                anim_start: 0,
                                anim_step: 0,
                            };
                            idx += 1;
                        }
                    }
                }
            }

            if let Some(activity_anim_id) = e.activity_anim {
                let g = self.data.sprite_graphics_item(self.data.activity_bubble_graphics());

                // Adjust top-left of 96x96 sprite to get top-left of 32x32 bubble
                let dest_x = dest_x + 32;
                let dest_y = dest_y - 10;
                // TODO: adjust X for left/right facing

                for &(cx, cy) in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
                    assert!(g.mirror == 0);
                    let dest_pos =
                        (dest_x + g.dest_offset.0 + cx * g.size.0,
                         dest_y + g.dest_offset.1 + cy * g.size.1);

                    buf[idx] = Vertex {
                        dest_pos: dest_pos,
                        src_pos: (g.src_offset.0 + cx * g.size.0,
                                  g.src_offset.1 + cy * g.size.1),
                        sheet: g.sheet,
                        color: (255, 255, 255),

                        ref_pos: (pos.x as u16,
                                  // TODO: hardcoded size
                                  // TODO: arbitrary adjustment
                                  pos.y as u16 + HACKY_ADJUSTMENT,
                                  pos.z as u16),
                        // TODO: hardcoded size
                        ref_size_z: 64,

                        anim_length: 1,
                        anim_rate: 1,
                        anim_start: (e.motion.start_time % 55440) as u16,
                        anim_step: 0,
                    };
                    idx += 1;
                }


                let a = self.data.animation(activity_anim_id);
                let l = self.data.activity_layer();
                let g = self.data.sprite_graphics_item(l.gfx_start + a.local_id);
                let dest_x = dest_x + 8;
                let dest_y = dest_y + 8;

                for &(cx, cy) in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
                    assert!(g.mirror == 0);
                    let dest_pos =
                        (dest_x + g.dest_offset.0 + cx * g.size.0,
                         dest_y + g.dest_offset.1 + cy * g.size.1);

                    buf[idx] = Vertex {
                        dest_pos: dest_pos,
                        src_pos: (g.src_offset.0 + cx * g.size.0,
                                  g.src_offset.1 + cy * g.size.1),
                        sheet: g.sheet,
                        color: (255, 255, 255),

                        ref_pos: (pos.x as u16,
                                  // TODO: hardcoded size
                                  // TODO: arbitrary adjustment
                                  pos.y as u16 + HACKY_ADJUSTMENT,
                                  pos.z as u16),
                        // TODO: hardcoded size
                        ref_size_z: 64,

                        anim_length: 1,
                        anim_rate: 1,
                        anim_start: (e.motion.start_time % 55440) as u16,
                        anim_step: 0,
                    };
                    idx += 1;
                }
            }
        }

        // Ran out of entites - we're done.
        (idx, false)
    }
}


fn count_quads_one(e: &Entity, is_pawn: bool) -> usize {
    count_layers(e.appearance) +
    if !is_pawn { name_len(&e.name) } else { 0 } +
    if e.activity_anim.is_some() { 2 } else { 0 }
}


// TODO: find a better place for these
pub const WINGS: u32 = 1 << 6;
pub const HORN: u32 = 1 << 7;
pub const STALLION: u32 = 1 << 8;
pub const LIGHT: u32 = 1 << 9;
pub const MANE_SHIFT: usize = 10;
pub const TAIL_SHIFT: usize = 13;
pub const EQUIP0_SHIFT: usize = 18;
pub const EQUIP1_SHIFT: usize = 22;
pub const EQUIP2_SHIFT: usize = 26;

pub static COLOR_TABLE: [u8; 6] = [0x00, 0x44, 0x88, 0xcc, 0xff, 0xff];

fn count_layers(appearance: u32) -> usize {
    let mut count = 1;  // base
    if appearance & WINGS != 0 { count += 2; }  // frontwing + backwing
    if appearance & HORN != 0 { count += 1; }   // horn
    count += 3;     // mane + tail + eyes
    if (appearance >> EQUIP0_SHIFT) & 0xf != 0 { count += 1; }  // equip0
    if (appearance >> EQUIP1_SHIFT) & 0xf != 0 { count += 1; }  // equip1
    // equip2 is not actually used yet.
    //if (appearance >> EQUIP2_SHIFT) & 0xf != 0 { count += 1; }  // equip2
    count
}

fn name_len(name: &Option<String>) -> usize {
    if let Some(ref name) = *name {
        fonts::NAME.iter_str(name).filter(|&(ref idx, _)| idx.is_some()).count()
    } else {
        0
    }
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
    let equip1 = (appearance >> EQUIP1_SHIFT) & 15;

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
    if equip1 != 0 {
        go(15 + equip1 as usize - 1, white);    // socks
    }
    if wings {
        go(2, tint1);
    }
    go(4, white); // eyes
    go(8 + tail as usize, tint0);
    go(5 + mane as usize, tint0);
    if horn {
        go(1, tint1);
    }
    if equip0 != 0 {
        go(11 + equip0 as usize - 1, white);
    }
}
