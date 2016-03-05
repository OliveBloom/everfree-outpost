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
use forest::common;
use forest::context::{Context, CaveDetailPass, CaveRampsPass};


pub const LAYER_SIZE: usize = ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) as usize;
pub const LAYER_BYTE_SIZE: usize = (LAYER_SIZE + 7) / 8;

pub struct CaveDetail {
    raw: [u8; LAYER_BYTE_SIZE],
}

impl CaveDetail {
    pub fn data(&self) -> &BitSlice {
        BitSlice::from_bytes(&self.raw)
    }

    pub fn data_mut(&mut self) -> &mut BitSlice {
        BitSlice::from_bytes_mut(&mut self.raw)
    }
}

impl Summary for CaveDetail {
    fn alloc() -> Box<CaveDetail> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        try!(f.write_bytes_slice(&self.raw));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<CaveDetail>> {
        let mut result = CaveDetail::alloc();
        try!(f.read_bytes_slice(&mut result.raw));
        Ok(result)
    }
}

impl common::GridLike for CaveDetail {
    type Elem = bool;

    fn spacing() -> i32 { CHUNK_SIZE }
    fn size() -> i32 { CHUNK_SIZE + 1 }

    fn get(&self, offset: V2) -> bool {
        self.data().get(Self::bounds().index(offset))
    }
}


pub fn generate(ctx: &mut Context,
                chunk: &mut CaveDetail,
                pid: Stable<PlaneId>,
                cpos: V2,
                layer: u8) {
    let mut grid = CellularGrid::new(scalar(CHUNK_SIZE * 3 + 1));
    let mut rng = ctx.get_rng(pid);

    // Init all chunks in `grid` using either `cave_detail` (if previously generated)
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        // These coordinates are relative to the origin of `grid`.
        let base = (offset + scalar(1)) * scalar(CHUNK_SIZE);
        let chunk_bounds = Region::new(base, base + scalar(CHUNK_SIZE + 1));

        if let Some(detail) = ctx.get_result::<CaveDetailPass>((pid, cpos + offset, layer)) {
            // Load previously-generated detail as fixed values.
            for pos in chunk_bounds.points() {
                grid.set_fixed(pos, detail.data().get(chunk_bounds.index(pos)));
            }
        }
    }

    let grid_bounds_global = grid.bounds() + (cpos - scalar(1)) * scalar(CHUNK_SIZE);
    // Bounds of affected area, relative to ramp position.
    let rel_bounds = Region::new(V2::new(0, 0), V2::new(2, 3)).expand(scalar(1));
    // Covers all ramps whose affected area overlaps the grid.
    let collect_bounds = Region::new(grid_bounds_global.min - rel_bounds.max + scalar(1),
                                     grid_bounds_global.max - rel_bounds.min);
    for r in &ctx.collect_points::<CaveRampsPass>(pid, collect_bounds) {
        if r.layer == layer {
            let wall_bounds = Region::new(r.pos + V2::new(0, 0), r.pos + V2::new(2, 1));
            let open_bounds = Region::new(r.pos + V2::new(0, 1), r.pos + V2::new(2, 3));

            for p in wall_bounds.intersect(grid_bounds_global).points() {
                grid.set_fixed(p - grid_bounds_global.min, true);
            }
            for p in open_bounds.intersect(grid_bounds_global).points() {
                grid.set_fixed(p - grid_bounds_global.min, false);
            }
        } else if r.layer + 1 == layer {
            let open_bounds = Region::new(r.pos + V2::new(0, 0),
                                          r.pos + V2::new(2, 3)).expand(scalar(1));
            for p in open_bounds.intersect(grid_bounds_global).points() {
                grid.set_fixed(p - grid_bounds_global.min, false);
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
    for pos in chunk_bounds.points() {
        chunk.data_mut().set(chunk_bounds.index(pos), grid.get(pos));
    }
}
