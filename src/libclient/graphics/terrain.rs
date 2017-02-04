use std::prelude::v1::*;

use data::BlockDef;
use graphics::types::BlockChunk;

use physics::v3::{V2, scalar, Region};
use physics::CHUNK_SIZE;

use graphics::ATLAS_SIZE;
use graphics::{IntrusiveCorner, GeometryGenerator};
use graphics::emit_quad;
use graphics::types::LocalChunks;
use platform::gl;
use terrain::{LOCAL_SIZE, LOCAL_MASK};


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


#[derive(Clone)]
pub struct GeomGen<'a> {
    block_data: &'a [BlockDef],
    chunk: &'a BlockChunk,
    cpos: V2,
}

impl<'a> GeomGen<'a> {
    pub fn new(block_data: &'a [BlockDef],
               chunk: &'a BlockChunk,
               cpos: V2) -> GeomGen<'a> {
        GeomGen {
            block_data: block_data,
            chunk: chunk,
            cpos: cpos,
        }
    }
}

impl<'a> GeometryGenerator for GeomGen<'a> {
    type Vertex = Vertex;

    fn generate<F: FnMut(Vertex)>(&mut self, mut emit: F) {
        let base = self.cpos.extend(0) * scalar(CHUNK_SIZE);
        let chunk_bounds = Region::sized(scalar(CHUNK_SIZE)) + base;
        for pos in chunk_bounds.points() {
            let block_idx = chunk_bounds.index(pos);
            let block_id = self.chunk[block_idx];
            let block = &self.block_data[block_id as usize];

            for side in 0 .. 4 {
                let tile = block.tile(side);
                if tile == 0 {
                    continue;
                }

                let s = tile % ATLAS_SIZE;
                let t = tile / ATLAS_SIZE;
                emit_quad(|v| emit(v), Vertex {
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
    }
}


#[derive(Clone)]
pub struct RegionGeomGen<'a> {
    block_data: &'a [BlockDef],
    chunks: &'a LocalChunks,
    bounds: Region<V2>,

}

impl<'a> RegionGeomGen<'a> {
    pub fn new(block_data: &'a [BlockDef],
               chunks: &'a LocalChunks,
               bounds: Region<V2>) -> RegionGeomGen<'a> {
        RegionGeomGen {
            block_data: block_data,
            chunks: chunks,
            bounds: bounds,
        }
    }
}

impl<'a> GeometryGenerator for RegionGeomGen<'a> {
    type Vertex = Vertex;

    fn generate<F: FnMut(Vertex)>(&mut self, mut emit: F) {
        let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE as i32));

        for cpos in self.bounds.points() {
            let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            let mut gen = GeomGen::new(self.block_data, &self.chunks[chunk_idx], cpos);
            gen.generate(|v| emit(v));
        }
    }

    fn count_verts(&self) -> usize {
        let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE as i32));
        let mut count = 0;

        for cpos in self.bounds.points() {
            let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            let gen = GeomGen::new(self.block_data, &self.chunks[chunk_idx], cpos);
            count += gen.count_verts();
        }

        count
    }
}

