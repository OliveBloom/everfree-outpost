use rand::Rng;

use libcommon_util::bytes::Bytes;
use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libterrain_gen_algo::disk_sampler::DiskSampler;

use forest::common::HasPos;
use forest::context::{Context, HeightDetailPass, RampPositionsPass};


pub const GRID_SIZE: i32 = 256;
pub const SPACING: i32 = 12;

define_points!(RampPositions: V2; GRID_SIZE);

pub fn generate_positions(ctx: &mut Context,
                          chunk: &mut RampPositions,
                          pid: Stable<PlaneId>,
                          gpos: V2) {
    let mut disk = DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING);
    let mut rng = ctx.get_rng(pid);

    // Init disk sampler with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);

        if let Some(detail) = ctx.get_result::<RampPositionsPass>((pid, gpos + offset)) {
            for &p in &detail.data {
                disk.add_init_point(p + base);
            }
        }
    }

    // Generate
    disk.generate(&mut rng, 20);

    // Save results into the `chunk`.
    let base = scalar::<V2>(GRID_SIZE);
    let bounds = Region::new(base, base + scalar(GRID_SIZE));
    for &pos in disk.points() {
        if !bounds.contains(pos) {
            continue;
        }
        chunk.data.push(pos - base);
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ramp {
    pub pos: V2,
    /// Indicates the *lower* of the two connected layers
    pub layer: u8,
}

impl HasPos for Ramp {
    fn pos(&self) -> V2 { self.pos }
    fn pos_mut(&mut self) -> &mut V2 { &mut self.pos }
}

unsafe impl Bytes for Ramp {}

define_points!(CaveRamps: Ramp; CHUNK_SIZE as usize);

const RAMP_SIZE: V2 = V2 { x: 2, y: 3 };

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveRamps,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let mut rng = ctx.get_rng(pid);

    let bounds = Region::new(cpos, cpos + scalar(1)) * scalar(CHUNK_SIZE);
    for &pos in &ctx.collect_points::<RampPositionsPass>(pid, bounds) {
        let ramp_bounds = Region::new(pos - scalar(1),
                                      pos + RAMP_SIZE + scalar(1));
        let layer = rng.gen_range(0, 7);
        let (at_top, above) = ctx.grid_fold::<HeightDetailPass,_,_>(
            pid, ramp_bounds, (0, 0), |(t, a), _, h| {
                if h == layer as i8 + 1 {
                    (t + 1, a)
                } else if h > layer as i8 + 1 {
                    (t, a + 1)
                } else {
                    (t, a)
                }
            });
        // The entire upper level should either be surface (`at_top`) or cave (`above`).
        if !(at_top == ramp_bounds.volume() || above == ramp_bounds.volume()) {
            continue;
        }

        chunk.data.push(Ramp {
            pos: pos - bounds.min,
            layer: layer,
        });
    }
}
