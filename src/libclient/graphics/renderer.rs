use std::prelude::v1::*;
use std::mem;
use std::slice;

use physics::v3::{V3, V2, scalar, Region};

use data::Data;
use gl::{self, GlContext, GlBuffer};
use graphics::GeometryGenerator;
use graphics::types::LocalChunks;
use terrain::{LOCAL_SIZE, LOCAL_MASK};
use util;

use super::terrain;


pub struct Renderer<GL: GlContext> {
    gl: GL,

    terrain_geom: GeomCache<GL, Region<V2>>,
}

impl<GL: GlContext> Renderer<GL> {
    pub fn new(mut gl: GL) -> Renderer<GL> {
        let terrain_geom = GeomCache::new(&mut gl);

        Renderer {
            gl: gl,

            terrain_geom: terrain_geom,
        }
    }

    pub fn update_terrain_geometry(&mut self,
                                   data: &Data,
                                   chunks: &LocalChunks,
                                   bounds: Region<V2>) {
        self.terrain_geom.update(bounds, |buffer, _| {
            let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE as i32));

            let mut vert_count = 0;
            for cpos in bounds.points() {
                let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
                vert_count += terrain::GeomGen::new(&data.blocks,
                                                    &chunks[chunk_idx],
                                                    cpos).count_verts();
            }

            buffer.alloc(vert_count * mem::size_of::<terrain::Vertex>());
            let mut buf = unsafe { util::zeroed_boxed_slice(64 * 1024) };

            let mut offset = 0;
            for cpos in bounds.points() {
                let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
                let mut gen = terrain::GeomGen::new(&data.blocks,
                                                    &chunks[chunk_idx],
                                                    cpos);
                load_buffer::<GL, _>(buffer, &mut gen, &mut buf, &mut offset);
            }
        });
    }

    pub fn invalidate_terrain_geometry(&mut self) {
        self.terrain_geom.invalidate();
    }

    pub fn get_terrain_buffer(&self) -> &GL::Buffer {
        self.terrain_geom.buffer()
    }
}


struct GeomCache<GL: GlContext, K: Eq> {
    buffer: GL::Buffer,
    last_key: Option<K>,
}

impl<GL: GlContext, K: Eq> GeomCache<GL, K> {
    pub fn new(gl: &mut GL) -> GeomCache<GL, K> {
        GeomCache {
            buffer: gl.create_buffer(),
            last_key: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.last_key = None;
    }

    pub fn is_valid(&self, k: &K) -> bool {
        if let Some(ref last_key) = self.last_key {
            last_key == k
        } else {
            false
        }
    }

    pub fn buffer(&self) -> &GL::Buffer {
        &self.buffer
    }

    pub fn update<F>(&mut self, k: K, f: F)
            where F: FnOnce(&mut GL::Buffer, &K) {
        if !self.is_valid(&k) {
            f(&mut self.buffer, &k);
            self.last_key = Some(k);
        }
    }
}


fn load_buffer<GL, G>(buf: &mut GL::Buffer,
                      gen: &mut G,
                      tmp: &mut [G::Vertex],
                      offset: &mut usize)
        where GL: GlContext, G: GeometryGenerator {
    let mut keep_going = true;
    while keep_going {
        let (len, more) = gen.generate(tmp);
        keep_going = more;

        let byte_len = len * mem::size_of::<G::Vertex>();
        let bytes = unsafe {
            slice::from_raw_parts(tmp.as_ptr() as *const u8, byte_len)
        };
        buf.load(*offset, bytes);
        *offset += byte_len;
    }
}


