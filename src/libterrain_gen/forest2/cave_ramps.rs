use std::fs::File;
use std::io;

use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::disk_sampler::DiskSampler;

use cache::Summary;
use forest2::context::Context;


pub struct CaveRamps {
    pub ramps: Vec<V2>,
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


pub const GRID_SIZE: i32 = 256;
pub const SPACING: i32 = 16;

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveRamps,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let mut disk = DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING);
    let mut rng = ctx.get_rng(pid);

    // Init disk sampler with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);

        if let Some(detail) = ctx.get_cave_ramps(pid, cpos + offset) {
            for &pos in &detail.ramps {
                disk.add_init_point(pos + base);
            }
        }
    }

    // Generate
    disk.generate(&mut rng, 20);

    // Save results into the `chunk`.
    let base = scalar::<V2>(GRID_SIZE);
    let bounds = Region::new(base, base + scalar(GRID_SIZE));
    for &pos in disk.points() {
        if bounds.contains(pos) {
            chunk.ramps.push(pos - bounds.min);
        }
    }
}

const RAMP_SIZE: V2 = V2 { x: 2, y: 3 };

pub fn ramps_in_region(ctx: &mut Context,
                       pid: Stable<PlaneId>,
                       bounds: Region<V2>) -> Vec<V2> {
    let expanded = Region::new(bounds.min - RAMP_SIZE + scalar(1), bounds.max);
    let grid_bounds = bounds.div_round(GRID_SIZE);
    let mut result = Vec::new();

    for gpos in grid_bounds.points() {
        let ramps = ctx.cave_ramps(pid, gpos);
        for &r in &ramps.ramps {
            let pos = r + gpos * scalar(GRID_SIZE);
            if expanded.contains(pos) {
                result.push(pos);
            }
        }
    }

    result
}
