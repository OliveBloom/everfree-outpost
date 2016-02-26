use std::fs::File;
use std::io;
use std::mem;

use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::perlin;

use cache::Summary;
use forest2::context::Context;


pub const HEIGHTMAP_SIZE: usize = 64;

pub struct HeightMap {
    pub buf: [i32; HEIGHTMAP_SIZE * HEIGHTMAP_SIZE],
}

impl Summary for HeightMap {
    fn alloc() -> Box<HeightMap> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        f.write_bytes_slice(&self.buf)
    }

    fn read_from(mut f: File) -> io::Result<Box<HeightMap>> {
        let mut result = HeightMap::alloc();
        try!(f.read_bytes_slice(&mut result.buf));
        Ok(result)
    }
}


pub fn generate(ctx: &mut Context,
                chunk: &mut HeightMap,
                pid: Stable<PlaneId>,
                pos: V2) {
    let seed = ctx.globals(pid).heightmap_seed;
    let coarse_params = perlin::Params {
        resolution: 64,
        offset: scalar(2),
        magnitude: 256,
        seed: seed,
    };
    let fine_params = perlin::Params {
        resolution: 4,
        offset: scalar(0),
        magnitude: 768,
        seed: seed,
    };

    let size = scalar(HEIGHTMAP_SIZE as i32);
    let bounds = Region::new(scalar(0), size);
    for offset in bounds.points() {
        let p = pos * size + offset;
        let val = perlin::sample(&coarse_params, p) + perlin::sample(&fine_params, p);
        chunk.buf[bounds.index(offset)] = val;
    }
}
