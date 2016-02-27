use std::fs::File;
use std::io;
use std::mem;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::bilinear;

use cache::Summary;
use forest2::context::Context;
use forest2::height_map;


pub struct HeightDetail {
    pub buf: [i32; ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) as usize],
}

impl Summary for HeightDetail {
    fn alloc() -> Box<HeightDetail> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        f.write_bytes_slice(&self.buf)
    }

    fn read_from(mut f: File) -> io::Result<Box<HeightDetail>> {
        let mut result = HeightDetail::alloc();
        try!(f.read_bytes_slice(&mut result.buf));
        Ok(result)
    }
}


pub fn generate(ctx: &mut Context,
                chunk: &mut HeightDetail,
                pid: Stable<PlaneId>,
                pos: V2) {
    let mut height_points = [0; 4 * 4];
    let height_bounds = Region::new(scalar(-1), scalar(3));
    height_map::fold_region(ctx, pid, height_bounds + pos, (), |_, p, val| {
        height_points[height_bounds.index(p - pos)] = val;
    });
    let height_points = height_points;

    let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));
    for p in bounds.points() {
        let h = bilinear(p, CHUNK_SIZE, |p| height_points[height_bounds.index(p)]);
        chunk.buf[bounds.index(p)] = h;
    }
}

pub fn fold_region<F, S>(ctx: &mut Context,
                         pid: Stable<PlaneId>,
                         bounds: Region<V2>,
                         init: S,
                         mut f: F) -> S
        where F: FnMut(S, V2, i32) -> S {
    let chunk_bounds = bounds.div_round_signed(CHUNK_SIZE);

    let mut state = init;
    for cpos in chunk_bounds.points() {
        let chunk = ctx.height_detail(pid, cpos);
        let chunk_bounds = Region::new(cpos, cpos + scalar(1)) * scalar(CHUNK_SIZE);
        for p in bounds.intersect(chunk_bounds).points() {
            let val = chunk.buf[chunk_bounds.index(p)];
            state = f(state, p, val);
        }
    }

    state
}
