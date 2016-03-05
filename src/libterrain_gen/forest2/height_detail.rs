use std::fs::File;
use std::io;
use std::mem;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};
use libterrain_gen_algo::bilinear;

use cache::Summary;
use forest2::context::{Context, HeightMapPass};
use forest2::height_map;


define_grid!(HeightDetail: i8; CHUNK_SIZE as usize; +1);


pub fn generate(ctx: &mut Context,
                chunk: &mut HeightDetail,
                pid: Stable<PlaneId>,
                pos: V2) {
    let mut height_points = [0; 4 * 4];
    let height_bounds = Region::new(scalar(-1), scalar(3));
    ctx.grid_fold::<HeightMapPass,_,_>(pid, height_bounds + pos, (), |_, p, val| {
        height_points[height_bounds.index(p - pos)] = val;
    });
    let height_points = height_points;

    let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));
    for p in bounds.points() {
        let h = bilinear(p, CHUNK_SIZE, |p| height_points[height_bounds.index(p)]);
        chunk.data[bounds.index(p)] =
            if h < -128 {
                -1
            } else if h < 0 {
                0
            } else if h < 256 {
                // 0 .. 255 maps to 0 .. 7
                (h / 32) as i8
            } else {
                7
            };
    }
}
