use std::cmp;
use std::fs::File;
use std::io;
use rand::Rng;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{Bytes, ReadBytes, WriteBytes};
use libterrain_gen_algo::disk_sampler::DiskSampler;

use cache::Summary;
use forest2::context::{Context, HeightMapPass};
use forest2::height_map;


pub struct CaveJunk {
    pub buf: Vec<(V2, u8)>,
}

impl Summary for CaveJunk {
    fn alloc() -> Box<CaveJunk> {
        Box::new(CaveJunk { buf:  Vec::new() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        try!(f.write_bytes(self.buf.len() as u32));
        try!(f.write_bytes_slice(&self.buf));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<CaveJunk>> {
        let len = try!(f.read_bytes::<u32>()) as usize;
        let mut result = CaveJunk::alloc();
        result.buf = Vec::with_capacity(len);
        unsafe {
            result.buf.set_len(len);
            try!(f.read_bytes_slice(&mut result.buf));
        }
        Ok(result)
    }
}


pub const GRID_SIZE: i32 = 64;
pub const SPACING: i32 = 3;


pub fn generate(ctx: &mut Context,
                chunk: &mut CaveJunk,
                pid: Stable<PlaneId>,
                gpos: V2) {
    let mut disks = Vec::new();
    for _ in 0 .. CHUNK_SIZE / 2 {
        disks.push(DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING));
    }
    let mut rng = ctx.get_rng(pid);

    // Init disk samplers with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);

        if let Some(detail) = ctx.get_cave_junk(pid, gpos + offset) {
            for &(pos, layer) in &detail.buf {
                disks[layer as usize].add_init_point(pos + base);
            }
        }
    }

    let global_bounds = Region::new(gpos, gpos + scalar(1)) * scalar(GRID_SIZE);
    let chunk_bounds = global_bounds.div_round_signed(CHUNK_SIZE);
    let max_layer = ctx.grid_fold::<HeightMapPass,_,_>(pid, chunk_bounds, 0, |acc, _, height| {
        let cur = cmp::max(0, cmp::min(7, height / 32)) as u8;
        cmp::max(acc, cur)
    });

    // Generate and save results
    let base = scalar::<V2>(GRID_SIZE);
    let bounds = Region::new(base, base + scalar(GRID_SIZE));
    for (layer, disk) in disks.iter_mut().enumerate() {
        let layer = layer as u8;
        if layer >= max_layer {
            // Don't generate for layers where there are no caves.
            continue;
        }

        disk.generate(&mut rng, 20);
        for &pos in disk.points() {
            if !bounds.contains(pos) {
                continue;
            }
            chunk.buf.push((pos - base, layer));
        }
    }
}

pub fn junk_in_region(ctx: &mut Context,
                      pid: Stable<PlaneId>,
                      bounds: Region<V2>) -> Vec<(V2, u8)> {
    // All objects have size 1x1x1, so no need to expand the bounds.
    let chunk_bounds = bounds.div_round_signed(CHUNK_SIZE);
    let mut result = Vec::new();

    for cpos in chunk_bounds.points() {
        let base = cpos * scalar(CHUNK_SIZE);
        let junk = ctx.cave_junk(pid, cpos);
        for &(pos, layer) in &junk.buf {
            if bounds.contains(pos + base) {
                result.push((pos + base, layer));
            }
        }
    }

    result
}
