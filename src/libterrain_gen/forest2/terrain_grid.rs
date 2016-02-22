use std::cmp;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libterrain_gen_algo::bilinear;

use forest2::context::{self, Context, TerrainGrid};

pub fn generate(ctx: &mut Context,
                chunk: &mut TerrainGrid,
                pid: Stable<PlaneId>,
                pos: V2) {
    let mut height_points = [0; 4 * 4];
    let height_bounds = Region::new(scalar(-1), scalar(3));
    for p in height_bounds.points() {
        height_points[height_bounds.index(p)] = ctx.point_height(pid, pos + p);
    }
    let height_points = height_points;

    let get_height = |pos| {
        bilinear(pos, CHUNK_SIZE, |p| height_points[height_bounds.index(p)])
    };

    let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));
    for p in bounds.points() {
        // Apply heightmap
        let h = cmp::min(
            cmp::min(get_height(p + V2::new(-1, -1)),
                     get_height(p + V2::new(-1,  0))),
            cmp::min(get_height(p + V2::new( 0, -1)),
                     get_height(p + V2::new( 0,  0))));
        let h =
            if h.abs() >= 256 {
                warn!("perlin noise value exceeds bounds: {} @ {:?}", h, p);
                255
            } else {
                h
            };
        let block_height = cmp::max(0, h / 32) as usize;
        for layer in 0 .. block_height {
            chunk.buf[layer][bounds.index(p)].flags.insert(context::T_FLOOR | context::T_WALL);
        }
        chunk.buf[block_height][bounds.index(p)].flags.insert(context::T_FLOOR);

        if h < -128 {
            chunk.buf[0][bounds.index(p)].flags.insert(context::T_WATER);
        }
    }
}
