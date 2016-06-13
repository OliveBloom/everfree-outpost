use std::collections::VecDeque;

use v3::{V3, V2, Region};

use super::{Shape, ShapeSource};


/// Check if the floor is blocked at this location.
fn check_floor<S: ShapeSource>(chunk: &S, pos: V3) -> bool {
    chunk.get_shape(pos) == Shape::Solid &&
    chunk.get_shape(pos + V3::new(0, 0, 1)) == Shape::Solid
}

/// Check if the ceiling is blocked at some point above this location.
fn check_ceiling<S: ShapeSource>(chunk: &S, pos: V3) -> bool {
    let base = pos.reduce();
    for z in pos.z + 2 .. 16 {
        if chunk.get_shape(base.extend(z)) != Shape::Empty {
            return true;
        }
    }
    false
}

/// Check if floodfilling should stop at this point.
fn stop_fill<S: ShapeSource>(chunk: &S, pos: V3) -> bool {
    // Stop if floor is blocked or if we've left the covered area.
    check_floor(chunk, pos) || !check_ceiling(chunk, pos)
}

pub mod flags {
    bitflags! {
        pub flags Flags: u8 {
            /// The cell has already been enqueued.  There's no separate flag for cells that have
            /// been fully processed, since each cell will be enqueued at most once.
            const ENQUEUED =    1 << 0,
            const INSIDE =      1 << 1,
            const INSIDE_NW =   1 << 2,
            const INSIDE_NE =   1 << 3,
            const INSIDE_SW =   1 << 4,
            const INSIDE_SE =   1 << 5,
            const ALL_CORNERS_INSIDE =
                INSIDE_NW.bits | INSIDE_NE.bits | INSIDE_SW.bits | INSIDE_SE.bits,
        }
    }
}

pub fn floodfill<S>(start: V3,
                    bounds: Region<V2>,
                    chunk: &S,
                    grid: &mut [flags::Flags],
                    grid_bounds: Region<V2>)
        where S: ShapeSource {
    let mut queue = VecDeque::new();
    queue.push_back(start.reduce());
    let z = start.z;
    while let Some(pos) = queue.pop_front() {
        if !stop_fill(chunk, pos.extend(z)) {
            grid[grid_bounds.index(pos)].insert(flags::INSIDE);

            let mut maybe_enqueue = |x, y| {
                let p = pos + V2::new(x, y);
                if !bounds.contains(p) {
                    return;
                }
                let idx = grid_bounds.index(p);
                if !grid[idx].contains(flags::ENQUEUED) {
                    grid[idx].insert(flags::ENQUEUED);
                    queue.push_back(p);
                }
            };

            maybe_enqueue(-1,  0);
            maybe_enqueue( 1,  0);
            maybe_enqueue( 0, -1);
            maybe_enqueue( 0,  1);
        }
    }

    // Populate corner-fill flags.
    for pos in grid_bounds.points() {
        let inside = |grid: &[flags::Flags], dx, dy| {
            let p = pos + V2::new(dx, dy);
            bounds.contains(p) &&
                grid[grid_bounds.index(p)].contains(flags::INSIDE)
        };

        if inside(grid, 0, 0) {
            grid[grid_bounds.index(pos)].insert(flags::ALL_CORNERS_INSIDE);
        } else {
            let n = inside(grid,  0, -1);
            let s = inside(grid,  0,  1);
            let w = inside(grid, -1,  0);
            let e = inside(grid,  1,  0);

            let nw = n || w || inside(grid, -1, -1);
            let ne = n || e || inside(grid,  1, -1);
            let sw = s || w || inside(grid, -1,  1);
            let se = s || e || inside(grid,  1,  1);

            grid[grid_bounds.index(pos)].insert(
                if nw { flags::INSIDE_NW } else { flags::Flags::empty() } |
                if ne { flags::INSIDE_NE } else { flags::Flags::empty() } |
                if sw { flags::INSIDE_SW } else { flags::Flags::empty() } |
                if se { flags::INSIDE_SE } else { flags::Flags::empty() });
        }
    }
}
