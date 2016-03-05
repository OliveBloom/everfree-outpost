use std::cmp;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::Bytes;
use libterrain_gen_algo::disk_sampler::DiskSampler;

use forest::common::HasPos;
use forest::context::{Context, HeightDetailPass, TreePositionsPass};


const GRID_SIZE: i32 = 64;
const SPACING: i32 = 4;

define_points!(TreePositions: V2; GRID_SIZE as usize);

pub fn generate_positions(ctx: &mut Context,
                          chunk: &mut TreePositions,
                          pid: Stable<PlaneId>,
                          cpos: V2) {
    let mut disk = DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING);
    let mut rng = ctx.get_rng(pid);

    // Init disk samplers with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);

        if let Some(detail) = ctx.get_result::<TreePositionsPass>((pid, cpos + offset)) {
            for &pos in &detail.data {
                disk.add_init_point(pos + base);
            }
        }
    }

    // Generate and save results
    disk.generate(&mut rng, 20);

    let base = scalar::<V2>(GRID_SIZE);
    let bounds = Region::new(base, base + scalar(GRID_SIZE));
    for &pos in disk.points() {
        if !bounds.contains(pos) {
            continue;
        }
        chunk.data.push(pos - base);
    }
}


define_points!(Trees: Tree; CHUNK_SIZE as usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tree {
    pub pos: V2,
    pub layer: u8,
}

unsafe impl Bytes for Tree {}

impl HasPos for Tree {
    fn pos(&self) -> V2 { self.pos }
    fn pos_mut(&mut self) -> &mut V2 { &mut self.pos }
}

pub fn generate(ctx: &mut Context,
                chunk: &mut Trees,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let bounds = Region::new(cpos, cpos + scalar(1)) * scalar(CHUNK_SIZE);
    let rel_bounds = Region::new(scalar(0), scalar(4));
    for &pos in &ctx.collect_points::<TreePositionsPass>(pid, bounds) {
        let (min, max) = ctx.grid_fold::<HeightDetailPass,_,_>(
            pid, rel_bounds + pos, (8, -1), |(min, max), _, h| {
                (cmp::min(min, h),
                 cmp::max(max, h))
            });

        if min != max || min == -1 {
            continue;
        }

        // Leave a clearing near spawn
        if pos.dot(pos) < 5 * 5 {
            continue;
        }

        chunk.data.push(Tree {
            pos: pos - bounds.min,
            layer: min as u8,
        });
    }
}
