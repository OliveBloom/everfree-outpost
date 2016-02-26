use std::fs::File;
use std::io;
use std::mem;
use rand::Rng;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::BitSlice;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::cellular::CellularGrid;

use cache::Summary;
use forest2::context::{self, Context};


pub const LAYER_SIZE: usize = ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) as usize;
pub const LAYER_BYTE_SIZE: usize = (LAYER_SIZE + 7) / 8;

pub type CaveDetailLayer = [u8; LAYER_BYTE_SIZE];

pub struct CaveDetail {
    buf: [CaveDetailLayer; CHUNK_SIZE as usize / 2],
}

impl CaveDetail {
    pub fn layer(&self, layer: usize) -> &BitSlice {
        BitSlice::from_bytes(&self.buf[layer])
    }

    pub fn layer_mut(&mut self, layer: usize) -> &mut BitSlice {
        BitSlice::from_bytes_mut(&mut self.buf[layer])
    }
}

impl Summary for CaveDetail {
    fn alloc() -> Box<CaveDetail> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        for layer in &self.buf {
            try!(f.write_bytes_slice(layer));
        }
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<CaveDetail>> {
        let mut result = CaveDetail::alloc();
        for layer in &mut result.buf {
            try!(f.read_bytes_slice(layer));
        }
        Ok(result)
    }
}


fn generate_layer(ctx: &mut Context,
                  chunk: &mut CaveDetail,
                  pid: Stable<PlaneId>,
                  cpos: V2,
                  layer: usize) {
    let mut grid = CellularGrid::new(scalar(CHUNK_SIZE * 3 + 1));
    let mut rng = ctx.get_rng(pid);

    // Init all chunks in `grid` using either `cave_detail` (if previously generated) or
    // `height_detail`.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        // These coordinates are relative to the origin of `grid`.
        let base = (offset + scalar(1)) * scalar(CHUNK_SIZE);
        let chunk_bounds = Region::new(base, base + scalar(CHUNK_SIZE + 1));

        if let Some(detail) = ctx.get_cave_detail(pid, cpos + offset) {
            // Load previously-generated detail as fixed values.
            let slice = detail.layer(layer);
            for pos in chunk_bounds.points() {
                grid.set_fixed(pos, slice.get(chunk_bounds.index(pos)));
            }
        }
    }

    // Fill remaining space at random.
    grid.init(|_| rng.gen_range(0, 10) < 5);

    // Iterate
    for _ in 0 .. 5 {
        grid.step(|here, active, total| 2 * (here as u8 + active) > total);
    }

    // Save results into the `chunk`.
    let base = scalar::<V2>(CHUNK_SIZE);
    let chunk_bounds = Region::new(base, base + scalar(CHUNK_SIZE + 1));
    let slice = chunk.layer_mut(layer);
    for pos in chunk_bounds.points() {
        slice.set(chunk_bounds.index(pos), grid.get(pos));
    }
}

pub fn generate(ctx: &mut Context,
                chunk: &mut CaveDetail,
                pid: Stable<PlaneId>,
                cpos: V2) {
    for layer in 0 .. CHUNK_SIZE as usize / 2 {
        generate_layer(ctx, chunk, pid, cpos, layer);
    }
}
