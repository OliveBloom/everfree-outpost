use std::fs::File;
use std::io;
use std::mem;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::bilinear;

use cache::Summary;
use forest2::context::Context;


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
    for p in height_bounds.points() {
        height_points[height_bounds.index(p)] = ctx.point_height(pid, pos + p);
    }
    let height_points = height_points;

    let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));
    for p in bounds.points() {
        let h = bilinear(p, CHUNK_SIZE, |p| height_points[height_bounds.index(p)]);
        chunk.buf[bounds.index(p)] = h;
    }
}
