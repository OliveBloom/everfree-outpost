use std::prelude::v1::*;

use data::BlockDef;
use graphics::types::BlockChunk;

use physics::v3::{V3, V2, scalar, Region, RegionPoints};
use physics::CHUNK_SIZE;

use graphics::ATLAS_SIZE;
use graphics::{IntrusiveCorner, GeometryGenerator};
use graphics::{emit_quad, remaining_quads};
use platform::gl;


/// Vertex attributes for terrain.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Vertex {
    corner: (u8, u8),
    pos: (u8, u8, u8),
    side: u8,
    tex_coord: (u8, u8),
}

impl IntrusiveCorner for Vertex {
    fn corner(&self) -> &(u8, u8) { &self.corner }
    fn corner_mut(&mut self) -> &mut (u8, u8) { &mut self.corner }
}

pub fn load_shader<GL: gl::Context>(gl: &mut GL) -> GL::Shader {
    gl.load_shader(
        "terrain2.vert", "terrain2.frag",
        defs! {
            SLICE_ENABLE: "1",
            SLICE_SIMPLIFIED: "1",
        },
        uniforms! {
            cameraPos: V2,
            cameraSize: V2,
            sliceCenter: V2,
            sliceZ: Float,
        },
        arrays! {
            [8] attribs! {
                corner: U8[2] @0,
                blockPos: U8[3] @2,
                side: U8[1] @5,
                tileCoord: U8[2] @6,
            },
        },
        textures! {
            atlasTex,
            cavernTex,
        },
        outputs! { color: 2, depth })
}


pub struct GeomGen<'a> {
    block_data: &'a [BlockDef],
    chunk: &'a BlockChunk,
    cpos: V2,
    iter: RegionPoints<V3>,
}

impl<'a> GeomGen<'a> {
    pub fn new(block_data: &'a [BlockDef],
               chunk: &'a BlockChunk,
               cpos: V2) -> GeomGen<'a> {
        GeomGen {
            block_data: block_data,
            chunk: chunk,
            cpos: cpos,
            iter: Region::new(scalar(0), scalar::<V3>(CHUNK_SIZE)).points(),
        }
    }

    pub fn count_verts(&self) -> usize {
        let mut count = 0;
        for &b in self.chunk.iter() {
            if b == 0 {
                continue;
            }

            let block_def = &self.block_data[b as usize];
            for side in 0 .. 4 {
                if block_def.tile(side) != 0 {
                    count += 6;
                }
            }
        }
        count
    }
}

impl<'a> GeometryGenerator for GeomGen<'a> {
    type Vertex = Vertex;

    fn generate(&mut self,
                buf: &mut [Vertex]) -> (usize, bool) {
        let mut idx = 0;
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
        while remaining_quads(buf, idx) >= 4 {
            let iter_pos = match self.iter.next() {
                Some(p) => p,
                None => return (idx, false),
            };

            let pos = self.cpos.extend(0) * scalar(CHUNK_SIZE) + iter_pos;

            let block_idx = chunk_bounds.index(iter_pos);
            let block_id = self.chunk[block_idx];
            let block = &self.block_data[block_id as usize];

            for side in 0 .. 4 {
                let tile = block.tile(side);
                if tile == 0 {
                    continue;
                }

                let s = tile % ATLAS_SIZE;
                let t = tile / ATLAS_SIZE;
                emit_quad(buf, &mut idx, Vertex {
                    corner: (0, 0),
                    pos: (pos.x as u8,
                          pos.y as u8,
                          pos.z as u8),
                    side: side as u8,
                    tex_coord: (s as u8,
                                t as u8),
                });
            }
        }

        // Stopped because the buffer is full.
        (idx, true)
    }
}
