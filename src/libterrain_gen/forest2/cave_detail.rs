use rand::Rng;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libterrain_gen_algo::cellular::CellularGrid;

use forest2::context::{self, Context, CaveDetail};

fn generate_layer(ctx: &mut Context,
                  chunk: &mut CaveDetail,
                  pid: Stable<PlaneId>,
                  cpos: V2,
                  layer: usize) {
    let mut grid = CellularGrid::new(scalar(CHUNK_SIZE * 3 + 1));
    let mut rng = ctx.get_rng(pid);

    // Init all chunks in `grid` using either `cave_detail` (if previously generated) or
    // `height_detail`.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        // These coordinates are relative to the origin of `grid`.
        let base = (offset + scalar(1)) * scalar(CHUNK_SIZE);
        let chunk_bounds = Region::new(base, base + scalar(CHUNK_SIZE + 1));

        if let Some(detail) = ctx.get_cave_detail(pid, cpos + offset) {
            // Load previously-generated detail as fixed values.
            let slice = detail.layer(layer);
            for pos in chunk_bounds.points() {
                grid.set_fixed(pos, slice.get(chunk_bounds.index(pos)));
            }
        }
    }

    // Fill remaining space at random.
    grid.init(|_| rng.gen_range(0, 10) < 5);

    // Iterate
    for _ in 0 .. 5 {
        grid.step(|here, active, total| 2 * (here as u8 + active) > total);
    }

    // Save results into the `chunk`.
    let base = scalar::<V2>(CHUNK_SIZE);
    let chunk_bounds = Region::new(base, base + scalar(CHUNK_SIZE + 1));
    let slice = chunk.layer_mut(layer);
    for pos in chunk_bounds.points() {
        slice.set(chunk_bounds.index(pos), grid.get(pos));
    }
}

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveDetail,
                pid: Stable<PlaneId>,
                cpos: V2) {
    for layer in 0 .. CHUNK_SIZE as usize / 2 {
        generate_layer(ctx, chunk, pid, cpos, layer);
    }
}
