use libserver_types::*;
use libterrain_gen_algo::perlin;

use forest::context::Context;


pub const HEIGHTMAP_SIZE: usize = 64;

define_grid!(HeightMap: i32; HEIGHTMAP_SIZE);

pub fn generate(ctx: &mut Context,
                chunk: &mut HeightMap,
                pid: Stable<PlaneId>,
                pos: V2) {
    let seed = ctx.globals(pid).heightmap_seed;
    let params = [
        // Primary hill/lake feature generation.  Produces hills up to height 3 (almost 4), with
        // feature spacing of about 90 seconds' run = 24 chunks.
        perlin::Params {
            resolution: 20,
            offset: scalar(0),
            magnitude: 256,
            seed: seed,
        },
        // Small-scale detail adjustment.  Adds a little noise to make hill/lake edges less smooth.
        perlin::Params {
            resolution: 4,
            offset: scalar(2),
            magnitude: 16,
            seed: seed,
        },
        /*
        perlin::Params {
            resolution: 4,
            offset: scalar(2),
            magnitude: 128,
            seed: seed,
        },
        */
    ];

    let size = scalar(HEIGHTMAP_SIZE as i32);
    let bounds = Region::new(scalar(0), size);
    for offset in bounds.points() {
        let p = pos * size + offset;
        let val: i32 = params.iter().map(|params| perlin::sample(&params, p)).sum();
        chunk.data[bounds.index(offset)] = val;
    }
}
