use std::fs::File;
use std::io;
use std::mem;

use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::perlin;

use cache::Summary;
use forest::context::Context;


pub const HEIGHTMAP_SIZE: usize = 64;

define_grid!(HeightMap: i32; HEIGHTMAP_SIZE);


pub fn generate(ctx: &mut Context,
                chunk: &mut HeightMap,
                pid: Stable<PlaneId>,
                pos: V2) {
    let seed = ctx.globals(pid).heightmap_seed;
    let params = [
        perlin::Params {
            resolution: 128,
            offset: scalar(8),
            magnitude: 256,
            seed: seed,
        },
        perlin::Params {
            resolution: 16,
            offset: scalar(0),
            magnitude: 256,
            seed: seed,
        },
        perlin::Params {
            resolution: 4,
            offset: scalar(2),
            magnitude: 128,
            seed: seed,
        },
    ];

    let size = scalar(HEIGHTMAP_SIZE as i32);
    let bounds = Region::new(scalar(0), size);
    for offset in bounds.points() {
        let p = pos * size + offset;
        let val: i32 = params.iter().map(|params| perlin::sample(&params, p)).sum();
        chunk.data[bounds.index(offset)] = val - 32;
    }
}
