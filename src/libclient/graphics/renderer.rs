use std::prelude::v1::*;
use std::mem;
use std::slice;

use physics::v3::{V3, V2, scalar, Region};

use data::Data;
use gl::{self, GlContext, GlBuffer};
use graphics::types::LocalChunks;
use terrain::{LOCAL_SIZE, LOCAL_MASK};
use util;

use super::terrain;


pub struct Renderer<GL: GlContext> {
    gl: GL,

    terrain_buffer: GL::Buffer,
}

impl<GL: GlContext> Renderer<GL> {
    pub fn new(mut gl: GL) -> Renderer<GL> {
        let terrain_buffer = gl.create_buffer();

        Renderer {
            gl: gl,

            terrain_buffer: terrain_buffer,
        }
    }

    pub fn update_terrain_geometry(&mut self,
                                   data: &Data,
                                   chunks: &LocalChunks,
                                   bounds: Region<V2>) {
        let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE as i32));

        let mut vert_count = 0;
        for cpos in bounds.points() {
            let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            vert_count += terrain::GeomGen::new(&data.blocks,
                                                &chunks[chunk_idx],
                                                cpos).count_verts();
        }

        self.terrain_buffer.alloc(vert_count * mem::size_of::<terrain::Vertex>());
        let mut buf = unsafe { util::zeroed_boxed_slice(64 * 1024) };

        let mut offset = 0;
        for cpos in bounds.points() {
            let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            let mut gen = terrain::GeomGen::new(&data.blocks,
                                                &chunks[chunk_idx],
                                                cpos);

            let mut more = true;
            while more {
                let mut idx = 0;
                more = gen.generate(&mut buf, &mut idx);

                let byte_len = idx * mem::size_of::<terrain::Vertex>();
                let bytes = unsafe {
                    slice::from_raw_parts(buf.as_ptr() as *const u8, byte_len)
                };
                self.terrain_buffer.load(offset, bytes);
                offset += byte_len;
            }
        }
    }

    pub fn get_terrain_buffer(&self) -> &GL::Buffer {
        &self.terrain_buffer
    }
}
