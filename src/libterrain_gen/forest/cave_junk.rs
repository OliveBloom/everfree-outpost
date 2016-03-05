use std::cmp;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libterrain_gen_algo::disk_sampler::DiskSampler;

use forest::context::{Context, HeightMapPass, CaveJunkPass};


define_points!(CaveJunk: V2; CHUNK_SIZE as usize);

pub const SPACING: i32 = 3;

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveJunk,
                pid: Stable<PlaneId>,
                cpos: V2,
                layer: u8) {
    let mut disk = DiskSampler::new(scalar(3 * CHUNK_SIZE), SPACING, 2 * SPACING);
    let mut rng = ctx.get_rng(pid);

    // Init disk samplers with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(CHUNK_SIZE);

        if let Some(detail) = ctx.get_result::<CaveJunkPass>((pid, cpos + offset, layer)) {
            for &pos in &detail.data {
                disk.add_init_point(pos + base);
            }
        }
    }

    let fold_bounds = Region::new(cpos, cpos + scalar(1));
    let max_layer = ctx.grid_fold::<HeightMapPass,_,_>(pid, fold_bounds, 0, |acc, _, height| {
        let cur = cmp::max(0, cmp::min(7, height / 32)) as u8;
        cmp::max(acc, cur)
    });

    if layer >= max_layer {
        // Don't generate anything for layers where there are no caves.
    }

    // Generate and save results
    disk.generate(&mut rng, 20);

    let base = scalar::<V2>(CHUNK_SIZE);
    let bounds = Region::new(base, base + scalar(CHUNK_SIZE));
    for &pos in disk.points() {
        if !bounds.contains(pos) {
            continue;
        }
        chunk.data.push(pos - base);
    }
}
