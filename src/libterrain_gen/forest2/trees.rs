use std::cmp;
use std::fs::File;
use std::io;
use rand::Rng;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{Bytes, ReadBytes, WriteBytes};
use libterrain_gen_algo::disk_sampler::DiskSampler;

use cache::Summary;
use forest2::context::Context;
use forest2::height_detail;


pub struct TreePositions {
    pub buf: Vec<V2>,
}

impl Summary for TreePositions {
    fn alloc() -> Box<TreePositions> {
        Box::new(TreePositions { buf:  Vec::new() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        try!(f.write_bytes(self.buf.len() as u32));
        try!(f.write_bytes_slice(&self.buf));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<TreePositions>> {
        let len = try!(f.read_bytes::<u32>()) as usize;
        let mut result = TreePositions::alloc();
        result.buf = Vec::with_capacity(len);
        unsafe {
            result.buf.set_len(len);
            try!(f.read_bytes_slice(&mut result.buf));
        }
        Ok(result)
    }
}


pub struct Trees {
    pub buf: Vec<(V2, u8)>,
}

impl Summary for Trees {
    fn alloc() -> Box<Trees> {
        Box::new(Trees { buf:  Vec::new() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        try!(f.write_bytes(self.buf.len() as u32));
        try!(f.write_bytes_slice(&self.buf));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<Trees>> {
        let len = try!(f.read_bytes::<u32>()) as usize;
        let mut result = Trees::alloc();
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


pub fn generate_positions(ctx: &mut Context,
                          chunk: &mut TreePositions,
                          pid: Stable<PlaneId>,
                          gpos: V2) {
    let mut disk = DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING);
    let mut rng = ctx.get_rng(pid);

    // Init disk samplers with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);

        if let Some(detail) = ctx.get_tree_positions(pid, gpos + offset) {
            for &pos in &detail.buf {
                disk.add_init_point(pos + base);
            }
        }
    }

    disk.generate(&mut rng, 20);

    // Generate and save results
    let base = scalar::<V2>(GRID_SIZE);
    let bounds = Region::new(base, base + scalar(GRID_SIZE));

    for &pos in disk.points() {
        if !bounds.contains(pos) {
            continue;
        }
        chunk.buf.push(pos - base);
    }
}

pub fn generate(ctx: &mut Context,
                chunk: &mut Trees,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let base = cpos * scalar(CHUNK_SIZE);
    let bounds = Region::new(base, base + scalar(CHUNK_SIZE));

    for &pos in &positions_in_region(ctx, pid, bounds) {
        let obj_bounds = Region::new(pos, pos + MAX_OBJECT_SIZE + scalar(1));
        let (min, max) = height_detail::fold_region(ctx, pid, obj_bounds, (7, 0), |(min, max), _, h| {
            let h =
                if h < -127 { -1 }
                else if h < 0 { 0 }
                else if h > 255 { 7 }
                else { h / 32 };
            (cmp::min(min, h), cmp::max(max, h))
        });
        if min < 0 || min != max {
            // Covered area is uneven or underwater
            continue;
        }
        chunk.buf.push((pos - base, min as u8));
    }
}

pub const MAX_OBJECT_SIZE: V2 = V2 { x: 3, y: 3 };

pub fn positions_in_region(ctx: &mut Context,
                           pid: Stable<PlaneId>,
                           bounds: Region<V2>) -> Vec<V2> {
    let expanded = Region::new(bounds.min - MAX_OBJECT_SIZE + scalar(1), bounds.max);
    let chunk_bounds = expanded.div_round_signed(GRID_SIZE);
    let mut result = Vec::new();

    for gpos in chunk_bounds.points() {
        let base = gpos * scalar(GRID_SIZE);
        let positions = ctx.tree_positions(pid, gpos);
        for &pos in &positions.buf {
            if expanded.contains(pos + base) {
                result.push(pos + base);
            }
        }
    }

    result
}

pub fn trees_in_region(ctx: &mut Context,
                       pid: Stable<PlaneId>,
                       bounds: Region<V2>) -> Vec<(V2, u8)> {
    let expanded = Region::new(bounds.min - MAX_OBJECT_SIZE + scalar(1), bounds.max);
    let chunk_bounds = expanded.div_round_signed(CHUNK_SIZE);
    let mut result = Vec::new();

    for cpos in chunk_bounds.points() {
        let base = cpos * scalar(CHUNK_SIZE);
        let positions = ctx.trees(pid, cpos);
        for &(pos, layer) in &positions.buf {
            if expanded.contains(pos + base) {
                result.push((pos + base, layer));
            }
        }
    }

    result
}
