use std::cmp;
use std::fs::File;
use std::io;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{Bytes, ReadBytes, WriteBytes};
use libterrain_gen_algo::disk_sampler::DiskSampler;

use cache::Summary;
use forest2::context::Context;
use forest2::height_detail;


pub struct RampPositions {
    /// Ramp positions, relative to the current grid chunk
    pub ramps: Vec<V2>,
}

impl Summary for RampPositions {
    fn alloc() -> Box<RampPositions> {
        Box::new(RampPositions { ramps:  Vec::new() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        try!(f.write_bytes(self.ramps.len() as u32));
        try!(f.write_bytes_slice(&self.ramps));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<RampPositions>> {
        let len = try!(f.read_bytes::<u32>()) as usize;
        let mut result = RampPositions::alloc();
        result.ramps = Vec::with_capacity(len);
        unsafe {
            result.ramps.set_len(len);
            try!(f.read_bytes_slice(&mut result.ramps));
        }
        Ok(result)
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


pub const GRID_SIZE: i32 = 256;
pub const SPACING: i32 = 16;

pub fn generate_positions(ctx: &mut Context,
                          chunk: &mut RampPositions,
                          pid: Stable<PlaneId>,
                          gpos: V2) {
    let mut disk = DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING);
    let mut rng = ctx.get_rng(pid);

    // Init disk sampler with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);

        if let Some(detail) = ctx.get_cave_ramp_positions(pid, gpos + offset) {
            for &p in &detail.ramps {
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
        chunk.ramps.push(pos - base);
        info!("  produced ramp @ {:?}", pos - base);
    }

    info!("produced {} ramps @ {:?}", chunk.ramps.len(), gpos);
}

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveRamps,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let bounds = Region::new(cpos, cpos + scalar(1)) * scalar(CHUNK_SIZE);
    info!("generating fine ramps at {:?} ({:?})", cpos, bounds);
    for &pos in &ramp_positions_in_region(ctx, pid, bounds) {
        let ramp_bounds = Region::new(pos - scalar(1),
                                      pos + RAMP_SIZE + scalar(1));
        let top_layer = height_detail::fold_region(ctx, pid, ramp_bounds, 8, |acc, _, h| {
            let cur = cmp::max(0, h) / 32;
            cmp::min(cur as u8, acc)
        });
        if top_layer == 0 || top_layer == 8 {
            continue;
        }

        chunk.ramps.push(Ramp {
            pos: pos - bounds.min,
            layer: top_layer - 1,
        });

        info!("  filtered {:?} -> {:?}", pos, pos - bounds.min);
    }
}

const RAMP_SIZE: V2 = V2 { x: 2, y: 3 };

pub fn ramp_positions_in_region(ctx: &mut Context,
                                pid: Stable<PlaneId>,
                                bounds: Region<V2>) -> Vec<V2> {
    let expanded = Region::new(bounds.min - RAMP_SIZE + scalar(1), bounds.max);
    let grid_bounds = bounds.div_round_signed(GRID_SIZE);
    let mut result = Vec::new();

    for gpos in grid_bounds.points() {
        let ramps = ctx.cave_ramp_positions(pid, gpos);
        for &p in &ramps.ramps {
            let pos = p + gpos * scalar(GRID_SIZE);
            if expanded.contains(pos) {
                result.push(pos);
            }
        }
    }

    result
}

pub fn ramps_in_region(ctx: &mut Context,
                       pid: Stable<PlaneId>,
                       bounds: Region<V2>) -> Vec<Ramp> {
    let expanded = Region::new(bounds.min - RAMP_SIZE + scalar(1), bounds.max);
    let chunk_bounds = bounds.div_round_signed(CHUNK_SIZE);
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
