use std::cmp;
use std::fs::File;
use std::io;
use rand::Rng;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{Bytes, ReadBytes, WriteBytes};
use libterrain_gen_algo::disk_sampler::DiskSampler;

use cache::Summary;
use forest2::common::HasPos;
use forest2::context::{Context, HeightDetailPass, RampPositionsPass};
use forest2::height_detail;


pub const GRID_SIZE: i32 = 256;
pub const SPACING: i32 = 16;

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

impl Ramp {
    fn offset(self, off: V2) -> Ramp {
        Ramp {
            pos: self.pos + off,
            layer: self.layer,
        }
    }
}

impl HasPos for Ramp {
    fn pos(&self) -> V2 { self.pos }
    fn pos_mut(&mut self) -> &mut V2 { &mut self.pos }
}

unsafe impl Bytes for Ramp {}

pub struct CaveRamps {
    pub ramps: Vec<Ramp>,
}

impl Summary for CaveRamps {
    fn alloc() -> Box<CaveRamps> {
        Box::new(CaveRamps { ramps:  Vec::new() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        try!(f.write_bytes(self.ramps.len() as u32));
        try!(f.write_bytes_slice(&self.ramps));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<CaveRamps>> {
        let len = try!(f.read_bytes::<u32>()) as usize;
        let mut result = CaveRamps::alloc();
        result.ramps = Vec::with_capacity(len);
        unsafe {
            result.ramps.set_len(len);
            try!(f.read_bytes_slice(&mut result.ramps));
        }
        Ok(result)
    }
}

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveRamps,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let mut rng = ctx.get_rng(pid);

    let bounds = Region::new(cpos, cpos + scalar(1)) * scalar(CHUNK_SIZE);
    info!("generating fine ramps at {:?} ({:?})", cpos, bounds);
    for &pos in &ctx.collect_points::<RampPositionsPass>(pid, bounds) {
        let ramp_bounds = Region::new(pos - scalar(1),
                                      pos + RAMP_SIZE + scalar(1));
        let layer = rng.gen_range(0, 7);
        let (at_top, above) = ctx.grid_fold::<HeightDetailPass,_,_>(
            pid, ramp_bounds, (0, 0), |(t, a), _, h| {
                let cur = cmp::max(0, h) / 32;
                if cur == layer as i32 + 1 {
                    (t + 1, a)
                } else if cur > layer as i32 + 1 {
                    (t, a + 1)
                } else {
                    (t, a)
                }
            });
        // The entire uppper level should either be surface (`at_top`) or cave (`above`).
        if !(at_top == ramp_bounds.volume() || above == ramp_bounds.volume()) {
            info!("  discarded {:?} z={} ({} {})", pos, layer, at_top, above);
            continue;
        }

        chunk.ramps.push(Ramp {
            pos: pos - bounds.min,
            layer: layer,
        });

        info!("  kept {:?} z={} ({} {})", pos, layer, at_top, above);
    }
}

const RAMP_SIZE: V2 = V2 { x: 2, y: 3 };

pub fn ramps_in_region(ctx: &mut Context,
                       pid: Stable<PlaneId>,
                       bounds: Region<V2>) -> Vec<Ramp> {
    let expanded = Region::new(bounds.min - RAMP_SIZE + scalar(1), bounds.max);
    let chunk_bounds = expanded.div_round_signed(CHUNK_SIZE);
    let mut result = Vec::new();

    for cpos in chunk_bounds.points() {
        let base = cpos * scalar(CHUNK_SIZE);
        let ramps = ctx.cave_ramps(pid, cpos);
        for r in &ramps.ramps {
            if expanded.contains(r.pos + base) {
                result.push(r.offset(base));
            }
        }
    }

    result
}
