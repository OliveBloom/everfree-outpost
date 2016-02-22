use libserver_types::*;
use libterrain_gen_algo::perlin;

use forest2::context::{Context, HeightMap, HEIGHTMAP_SIZE};

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
