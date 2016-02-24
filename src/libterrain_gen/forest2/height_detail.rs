use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libterrain_gen_algo::bilinear;

use forest2::context::{self, Context, HeightDetail};

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
