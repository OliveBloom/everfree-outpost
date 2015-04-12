#![crate_name = "graphics"]
#![no_std]

#![feature(no_std)]
#![feature(core)]

#![allow(unsigned_negation)]

#[macro_use] extern crate core;
#[cfg(asmjs)] #[macro_use] extern crate asmrt;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
extern crate physics;

use core::prelude::*;
use core::ptr;
use core::num::wrapping::WrappingOps;

use physics::{TILE_BITS, CHUNK_BITS};
use physics::v3::{V3, scalar, Region};


#[cfg(asmjs)]
mod std {
    pub use core::cmp;
    pub use core::fmt;
    pub use core::marker;
}


const ATLAS_SIZE: u16 = 32;

const TILE_SIZE: u16 = 1 << TILE_BITS;
const CHUNK_SIZE: u16 = 1 << CHUNK_BITS;

const LOCAL_BITS: usize = 3;
const LOCAL_SIZE: u16 = 1 << LOCAL_BITS;


/// Tile numbers used to display a particular block.
#[derive(Copy)]
pub struct BlockDisplay {
    pub front: u16,
    pub back: u16,
    pub top: u16,
    pub bottom: u16,
}

impl BlockDisplay {
    pub fn tile(&self, side: usize) -> u16 {
        match side {
            0 => self.front,
            1 => self.back,
            2 => self.top,
            3 => self.bottom,
            _ => panic!("invalid side number"),
        }
    }
}

/// BlockDisplay for every block type known to the client.
pub type BlockData = [BlockDisplay; (ATLAS_SIZE * ATLAS_SIZE) as usize];


/// A chunk of terrain.  Each element is a block index.
pub type BlockChunk = [u16; 1 << (3 * CHUNK_BITS)];
/// BlockChunk for every chunk in the local region.
pub type LocalChunks = [BlockChunk; 1 << (2 * LOCAL_BITS)];

/// Copy a BlockChunk into the LocalChunks.
pub fn load_chunk(local: &mut LocalChunks,
                  chunk: &BlockChunk,
                  cx: u16,
                  cy: u16) {
    let cx = cx & (LOCAL_SIZE - 1);
    let cy = cy & (LOCAL_SIZE - 1);
    let idx = cy * LOCAL_SIZE + cx;

    local[idx as usize] = *chunk;
}


/// Vertex attributes for terrain.
#[allow(dead_code)]
#[derive(Copy)]
pub struct TerrainVertex {
    x: u8,
    y: u8,
    z: u8,
    _pad0: u8,
    s: u8,
    t: u8,
    _pad1: u8,
    _pad2: u8,
}

/// Maximum number of blocks that could be present in the output of generate_geometry.  The +1 is
/// because generate_geometry actually looks at `CHUNK_SIZE + 1` y-positions.
const GEOM_BLOCK_COUNT: u16 = CHUNK_SIZE * (CHUNK_SIZE + 1) * CHUNK_SIZE;
/// Number of vertices in each face.
const FACE_VERTS: usize = 6;
/// A list of TerrainVertex items to pass to OpenGL.  The upper bound is GEOM_BLOCK_COUNT blocks *
/// 4 faces (sides) per block * FACE_VERTS vertices per face, but usually not all elements will be
/// filled in.
pub type TerrainGeometryBuffer = [TerrainVertex; GEOM_BLOCK_COUNT as usize * 4 * FACE_VERTS];

/// Generate terrain geometry for a chunk.  The result contains all faces that might overlap the
/// z=0 plane of the indicated chunk.  This means it contains the +y,-z half of the named chunk and
/// the -y,+z half of the next chunk in the +y direction.
///
/// This can actually output blocks at `CHUNK_SIZE + 1` y-positions for a particular x,z.  Only
/// CHUNK_SIZE blocks are actually visible, but those CHUNK_SIZE blocks include a half-block at the
/// top and a half-block at the bottom.  This function doesn't bother splitting back/top from
/// front/bottom, and just outputs the whole block on each end.
///
///         /-------/ CHUNK_SIZE visible (diagonal slice)
///      *-+-+-+-+-+
///      |/| | | |/|
///      +-+-+-+-+-*
///      |---------| CHUNK_SIZE + 1 output
/// Corners marked * are output even though they aren't actually visible.
pub fn generate_geometry<F>(local: &LocalChunks,
                            block_data: &BlockData,
                            geom: &mut TerrainGeometryBuffer,
                            cx: u16,
                            cy: u16,
                            filter: F) -> usize
        where F: Fn(V3, u16, &BlockDisplay) -> bool {

    let cx = cx & (LOCAL_SIZE - 1);
    let cy0 = cy & (LOCAL_SIZE - 1);
    let cy1 = (cy + 1) & (LOCAL_SIZE - 1);

    let bounds = Region::new(scalar(0), scalar(CHUNK_SIZE as i32));

    let mut out_idx = 0;

    const SIDE_OFFSETS: [((u8, u8), (u8, u8)); 4] = [
        // Front
        ((1, 1), (0, 1)),
        // Back
        ((0, 1), (0, 1)),
        // Top
        ((0, 1), (1, 0)),
        // Bottom
        ((0, 0), (1, 0)),
    ];

    const CORNERS: [(u8, u8); 4] = [
        (0, 0),
        (1, 0),
        (1, 1),
        (0, 1),
    ];

    const INDEXES: [usize; 6] = [0, 1, 2,  0, 2, 3];

    fn place(buf: &mut TerrainGeometryBuffer,
             base_idx: &mut usize,
             tile_id: u16,
             bx: i32,
             by: i32,
             bz: i32,
             side: usize) {
        let ((base_y, base_z), (step_y, step_z)) = SIDE_OFFSETS[side];

        let tile_s = (tile_id % ATLAS_SIZE) as u8;
        let tile_t = (tile_id / ATLAS_SIZE) as u8;

        for &idx in INDEXES.iter() {
            let (corner_u, corner_v) = CORNERS[idx];
            let vert = TerrainVertex {
                x: bx as u8 + corner_u,
                y: by as u8 + base_y + (step_y & corner_v),
                z: bz as u8 + base_z - (step_z & corner_v),
                _pad0: 0,
                s: tile_s + corner_u,
                t: tile_t + corner_v,
                _pad1: 0,
                _pad2: 0,
            };
            buf[*base_idx] = vert;
            *base_idx += 1;
        }
    }

    let chunk0 = &local[(cy0 * LOCAL_SIZE + cx) as usize];
    for z in range(0, CHUNK_SIZE as i32) {
        for y in range(z, CHUNK_SIZE as i32) {
            for x in range(0, CHUNK_SIZE as i32) {
                let block_id = chunk0[bounds.index(V3::new(x, y, z))];
                let data = &block_data[block_id as usize];

                if !filter(V3::new(x, y, z), block_id, data) {
                    continue;
                }

                for side in range(0, 4) {
                    let tile_id = data.tile(side);
                    if tile_id == 0 {
                        continue;
                    }
                    place(geom, &mut out_idx, tile_id, x, y, z, side);
                }
            }
        }
    }

    let chunk1 = &local[(cy1 * LOCAL_SIZE + cx) as usize];
    for z in range(0, CHUNK_SIZE as i32) {
        for y in range(0, z + 1) {  // NB: 0..z+1 instead of z..SIZE
            for x in range(0, CHUNK_SIZE as i32) {
                let block_id = chunk1[bounds.index(V3::new(x, y, z))];
                let data = &block_data[block_id as usize];

                if !filter(V3::new(x, y + 16, z), block_id, data) {
                    continue;
                }

                for side in range(0, 4) {
                    let tile_id = data.tile(side);
                    if tile_id == 0 {
                        continue;
                    }
                    place(geom, &mut out_idx, tile_id, x, y + 16, z, side);  // NB: +16
                }
            }
        }
    }

    out_idx
}


pub struct StructureTemplate {
    pub size: (u8, u8, u8),
    pub sheet: u8,
    pub display_size: (u16, u16),
    pub display_offset: (u16, u16),
}

/// All structure templates known to the client.  The number of elements is arbitrary.
pub type StructureTemplateData = [StructureTemplate; 1024];


pub struct Structure {
    live: bool,

    /// Structure position in tiles.  u8 is enough to cover the entire local region.
    pub pos: (u8, u8, u8),

    pub template_id: u16,
}

pub struct StructureBuffer<'a> {
    templates: &'a StructureTemplateData,

    /// Buffer containing all structures known to the client.  The limit is arbitrary, but 16 bits'
    /// worth seems reasonable and happens to be 1/8 of the theoretical maximum (if every block in
    /// the local region had a structure on both layers).
    structures: [Structure; 1 << 16],

    /// Index of the first empty slot in `structures`.
    first_free: usize,

    /// Index of the last non-empty slot in `structures`.
    last_used: usize,


    // Additional data used during geometry generation.

    /// Indexes of structures that overlap the target chunk.  Limit is arbitrary.  We may need to
    /// make multiple passes over this list to emit geometry for different sheets, which is why
    /// it's saved between calls.
    indexes: [u16; 1024],

    /// Index of the next structure to check once everything in `indexes` is done.  We save this
    /// separately (instead of just taking the last value in `indexes`) because we remove items
    /// from `indexes` as we output geometry.
    next_index: usize,

    /// Number of slots in `indexes` that are actually populated.
    num_indexes: usize,

    /// Bitfield of sheet numbers that are present in `indexes`.
    index_sheets: u32,
}

impl<'a> StructureBuffer<'a> {
    pub fn init(&mut self, templates: &'a StructureTemplateData) {
        unsafe { ptr::write(&mut self.templates, templates) };
        self.first_free = 0;
        self.last_used = 0;
        self.next_index = 0;
        self.num_indexes = 0;
        self.index_sheets = 0;
    }

    pub fn add_structure(&mut self, pos: (u8, u8, u8), template_id: u32) -> usize {
        let idx = {
            let s = &mut self.structures[self.first_free];
            s.live = true;
            s.pos = pos;
            s.template_id = template_id as u16;
            self.first_free
        };
        while self.first_free < self.structures.len() && self.structures[self.first_free].live {
            self.first_free += 1;
        }
        if idx > self.last_used {
            self.last_used = idx;
        }
        idx
    }

    pub fn remove_structure(&mut self, idx: usize) {
        self.structures[idx].live = false;
        while self.last_used > 0 && !self.structures[self.last_used].live {
            self.last_used -= 1;
        }
        if idx < self.first_free {
            self.first_free = idx;
        }
    }

    pub fn start_geometry_gen(&mut self) {
        self.next_index = 0;
        self.num_indexes = 0;
        self.index_sheets = 0;
    }

    pub fn continue_geometry_gen<F>(&mut self,
                                    buf: &mut StructureGeometryBuffer,
                                    cx: u8,
                                    cy: u8,
                                    filter: F) -> (usize, u8, bool)
            where F: Fn(&Structure, &StructureTemplate) -> bool {
        if self.num_indexes == 0 {
            self.fill_indexes(cx, cy, filter);
        }

        let (vertex_count, sheet) = self.generate_geometry(buf, cx, cy);
        let more = self.index_sheets != 0 || self.next_index <= self.last_used;
        (vertex_count, sheet, more)
    }

    fn fill_indexes<F>(&mut self, cx: u8, cy: u8, filter: F)
            where F: Fn(&Structure, &StructureTemplate) -> bool {
        // Most arithmetic in this function is wrapping arithmetic mod `LOCAL_SIZE * CHUNK_SIZE`.
        const MASK: u8 = (LOCAL_SIZE * CHUNK_SIZE - 1) as u8;

        fn add_wrap(a: u8, b: u8) -> u8 {
            a.wrapping_add(b) & MASK
        }

        fn sub_wrap(a: u8, b: u8) -> u8 {
            a.wrapping_sub(b) & MASK
        }


        const CHUNK_SIZE_U8: u8 = CHUNK_SIZE as u8;
        let min_x = sub_wrap(cx * CHUNK_SIZE_U8, CHUNK_SIZE_U8);
        let min_y = sub_wrap(cy * CHUNK_SIZE_U8, CHUNK_SIZE_U8);

        let range_x = CHUNK_SIZE_U8 * 2;
        let range_y = CHUNK_SIZE_U8 * 3;

        for idx in range(self.next_index, self.last_used + 1) {
            let s = &self.structures[idx];
            if !s.live {
                continue;
            }


            // Broad phase: based only on the structure's position, filter out structures that
            // definitely are not visible.
            let (x, y, z) = s.pos;
            let dx = sub_wrap(x, min_x);
            let dy = sub_wrap(y, min_y);
            if dx >= range_x || dy >= range_y {
                continue;
            }


            // Narrow phase: look at the size of the structure template to determine whether parts
            // of it might fall in the visible x,v region.
            let t = &self.templates[s.template_id as usize];
            let (sx, sy, sz) = t.size;
            // v-coordinate of the structure's reference point (-x,-y,-z corner).
            let v0 = sub_wrap(y, z);
            // Minimum v-coordinate of the structure's display.  With sz=(, the display extends 1
            // tile above v0, due to the top/back portion of the -y,-z row of blocks.  Every
            // additional sz causes it to extend further in the -v direction.
            let v = sub_wrap(v0, sz);

            // The structure's maximum size is CHUNK_SIZE on each axis.  This means that if the
            // structure does overlap the chunk, then at least one of {x, x + sx - 1} must be within
            // the chunk along the x axis, and at least one of {v, v0, v + sv - 1} must be within the
            // chunk along the v axis.
            let x_left = x;
            let x_right = add_wrap(x, sx - 1);
            let v_top = v;
            let v_middle = v0;
            let v_bottom = add_wrap(v, sy + sz - 1);
            let base_x = cx * CHUNK_SIZE_U8;
            let base_v = cy * CHUNK_SIZE_U8;
            if !((sub_wrap(x_left, base_x) < CHUNK_SIZE_U8 ||
                  sub_wrap(x_right, base_x) < CHUNK_SIZE_U8) &&
                 (sub_wrap(v_top, base_v) < CHUNK_SIZE_U8 ||
                  sub_wrap(v_middle, base_v) < CHUNK_SIZE_U8 ||
                  sub_wrap(v_bottom, base_v) < CHUNK_SIZE_U8)) {
                continue;
            }


            if !filter(s, t) {
                continue;
            }


            // The structure is definitely within the chunk.
            if self.num_indexes == self.indexes.len() {
                self.next_index = idx;
                break;
            } else {
                self.indexes[self.num_indexes] = idx as u16;
                self.num_indexes += 1;
                self.index_sheets |= 1 << t.sheet as usize;
            }
        }

        self.next_index = self.last_used + 1;
    }

    fn generate_geometry(&mut self,
                         buf: &mut StructureGeometryBuffer,
                         cx: u8,
                         cy: u8) -> (usize, u8) {
        let mut sheet = 0;
        for i in range(0, 32) {
            if self.index_sheets & (1 << i) != 0 {
                sheet = i;
                break;
            }
        }
        let sheet = sheet;

        let mut out_idx = 0;
        let mut buf_idx = 0;

        fn emit(buf: &mut StructureGeometryBuffer,
                idx: &mut usize,
                pos: (i16, i16, i16),
                base_z: i16,
                tex_coord: (u16, u16)) {
            let (x, y, z) = pos;
            buf[*idx].x = x;
            buf[*idx].y = y;
            buf[*idx].z = z;
            buf[*idx].base_z = base_z;
            let (s, t) = tex_coord;
            buf[*idx].s = s;
            buf[*idx].t = t;
            *idx += 1;
        }

        fn tile_to_px(tile: u8) -> i16 {
            return tile as i16 * TILE_SIZE as i16;
        }

        let base_x = tile_to_px(cx * CHUNK_SIZE as u8);
        let base_y = tile_to_px(cy * CHUNK_SIZE as u8);

        // Should be at least the maximum structure size, and no more than (local region size -
        // chunk size - max structure size).
        const MARGIN: i16 = (CHUNK_SIZE * TILE_SIZE) as i16;
        let origin_x = base_x - MARGIN;
        let origin_y = base_y - MARGIN;
        const MASK: i16 = (LOCAL_SIZE * CHUNK_SIZE * TILE_SIZE - 1) as i16;

        // Subtract (base_x, base_y) from (x, y), but wrap coordinates across the local region
        // borders so that (parts of) structures in the top chunk can appear in the bottom one and
        // vice versa.
        //
        //  +-----+    MARGIN   (remainder, may exceed MARGIN)
        //  |     |        /-\ /-\
        //  |     |        +-----+
        //  |     |        |     |
        //  |   +-+  wrap  | +-+ |
        //  |   | |   =>   | | | |
        //  +-----+        | +-+ |
        //                 |     |
        //                 +-----+
        let sub_base_wrap = |x, y| {
            (((x - origin_x) & MASK) - MARGIN,
             ((y - origin_y) & MASK) - MARGIN)
        };

        const CORNERS: [(u8, u8); 6] = [(0,0), (1,0), (1,1),  (0,0), (1, 1), (0,1)];

        // Walk through self.indexes, looking for structures whose template is on the correct
        // sheet.  Structures on other sheets get moved to the front of the list (over the top of
        // the matching structures, which need no futher processing).
        for in_idx in range(0, self.num_indexes) {
            let s = &self.structures[self.indexes[in_idx] as usize];
            let t = &self.templates[s.template_id as usize];

            if t.sheet != sheet {
                self.indexes[out_idx] = self.indexes[in_idx];
                out_idx += 1;
                continue;
            }

            let (x, y, z) = s.pos;
            let (_sx, sy, _sz) = t.size;

            // Do the rendering at the front (+y) side of the structure.
            let (px_x, px_y) = sub_base_wrap(tile_to_px(x), tile_to_px(y) + tile_to_px(sy));
            let px_z = tile_to_px(z);

            let (base_s, base_t) = t.display_offset;
            let (step_s, step_t) = t.display_size;

            for &(dx, dy) in CORNERS.iter() {
                let off_x = dx as i16 * step_s as i16;
                let off_y = 0;
                let off_z = dy as i16 * step_t as i16;
                let off_s = dx as u16 * step_s;
                let off_t = (1 - dy) as u16 * step_t;

                emit(buf, &mut buf_idx,
                     (px_x + off_x,
                      px_y + off_y,
                      px_z + off_z),
                     px_z,
                     (base_s + off_s,
                      base_t + off_t));
            }
        }

        self.num_indexes = out_idx;
        self.index_sheets &= !(1 << sheet);
        (buf_idx, sheet)
    }
}


/// Vertex attributes for structure rendering.
pub struct StructureVertex {
    x: i16,
    y: i16,
    z: i16,
    base_z: i16,
    s: u16,
    t: u16,
    _pad1: u16,
    _pad2: u16,
}

/// Buffer for StructureVertex items.  The number of elements is set to 6 times the length of
/// StructureBuffer.indexes.
pub type StructureGeometryBuffer = [StructureVertex; 6 * 1024];
