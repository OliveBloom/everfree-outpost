use std::cmp;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libterrain_gen_algo::bilinear;

use forest2::context::{self, Context, TerrainGrid};

pub fn generate(ctx: &mut Context,
                chunk: &mut TerrainGrid,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));

    // Apply heightmap info
    {
        let detail = ctx.height_detail(pid, cpos);
        for p in bounds.points() {
            let h = detail.buf[bounds.index(p)];
            let h =
                if h.abs() >= 256 {
                    warn!("perlin noise value exceeds bounds: {} @ {:?}", h, p);
                    255 * h.signum()
                } else {
                    h
                };
            let block_height = cmp::max(0, h / 32) as usize;
            let idx = bounds.index(p);
            for layer in 0 .. block_height {
                chunk.buf[layer][idx].flags.insert(context::T_CAVE | context::T_FLOOR);
                chunk.buf[layer][idx].floor_type = context::FloorType::Cave;
            }
            chunk.buf[block_height][idx].flags.insert(context::T_FLOOR);

            if h < -128 {
                chunk.buf[0][idx].floor_type = context::FloorType::Water;
            }
        }
    }

    // Apply cave info
    {
        let detail = ctx.cave_detail(pid, cpos);
        for layer in 0 .. CHUNK_SIZE as usize / 2 {
            for p in bounds.points() {
                if detail.layer(layer).get(bounds.index(p)) {
                    // It's a wall.
                    continue;
                }

                if chunk.buf[layer][bounds.index(p)].flags.contains(context::T_CAVE) {
                    chunk.buf[layer][bounds.index(p)].flags.insert(context::T_CAVE_INSIDE);
                }
            }
        }
    }
}
