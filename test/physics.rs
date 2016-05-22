extern crate physics;

use std::iter;

use physics::v3::{V3, V2, Vn, Region, scalar};
use physics::{Shape, ShapeSource};
use physics::walk2::Collider;


struct ShapeSourceImpl {
    buf: Box<[Shape]>,
    bounds: Region<V3>,
}

impl ShapeSource for ShapeSourceImpl {
    fn get_shape(&self, pos: V3) -> Shape {
        if !self.bounds.contains(pos) {
            return Shape::Empty;
        }

        self.buf[self.bounds.index(pos)]
    }
}

fn init_map(repr: &[&str]) -> ShapeSourceImpl {
    let h = repr.len();
    let w = if h > 0 { repr[0].len() } else { 0 };

    let size = V2::new(w as i32, h as i32);
    let bounds = Region::sized(size);
    let mut buf = iter::repeat(Shape::Empty).take(bounds.volume() as usize)
                      .collect::<Vec<_>>().into_boxed_slice();

    for (y, row) in repr.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let pos = V2::new(x as i32, y as i32);
            buf[bounds.index(pos)] = match c {
                ' ' => Shape::Empty,
                '.' => Shape::Floor,
                '#' => Shape::Solid,
                '^' => Shape::RampN,
                _ => panic!("bad char: {:?}", c),
            };
        }
    }

    ShapeSourceImpl {
        buf: buf,
        bounds: bounds.extend(0, 1),
    }
}

fn init_side_view(repr: &[&str], depth: i32) -> ShapeSourceImpl{ 
    let h = repr.len();
    let w = if h > 0 { repr[0].len() } else { 0 };

    let size = V3::new(depth + 2, w as i32, h as i32);
    let bounds = Region::sized(size);
    let mut buf = iter::repeat(Shape::Empty).take(bounds.volume() as usize)
                      .collect::<Vec<_>>().into_boxed_slice();

    for (inv_z, row) in repr.iter().enumerate() {
        let z = size.z - 1 - inv_z as i32;
        for (inv_y, c) in row.chars().enumerate() {
            let y = size.y - 1 - inv_y as i32;
            for x in 1 .. depth + 1 {
                let pos = V3::new(x, y, z);
                buf[bounds.index(pos)] = match c {
                    ' ' => Shape::Empty,
                    '_' => Shape::Floor,
                    '#' => Shape::Solid,
                    '/' => Shape::RampN,
                    _ => panic!("bad char: {:?}", c),
                };
            }
        }
    }

    for y in 0 .. size.y {
        buf[bounds.index(V3::new(0, y, 0))] = Shape::Floor;
        buf[bounds.index(V3::new(depth + 1, y, 0))] = Shape::Floor;
    }

    ShapeSourceImpl {
        buf: buf,
        bounds: bounds,
    }
}


#[test]
fn calc_velocity_basics() {
    let s = init_map(&[
        ".....",
        ".#...",
        ".....",
        ". ...",
        ".....",
    ]);

    let b = Region::sized(V3::new(32, 32, 48));

    // Walk in +x direction (not blocked)
    assert_eq!(
        Collider::new(&s, b).calc_velocity(V3::new(10, 0, 0)),
        V3::new(10, 0, 0));

    // Walk in +y direction (not blocked)
    assert_eq!(
        Collider::new(&s, b).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, 0));

    // Walk in +x+y direction (blocked by Solid)
    assert_eq!(
        Collider::new(&s, b).calc_velocity(V3::new(10, 10, 0)),
        V3::new(0, 0, 0));
}

#[test]
fn calc_velocity_sliding() {
    let s = init_map(&[
        ".....",
        ".#...",
        ".....",
        ". ...",
        ".....",
    ]);

    let b = Region::sized(V3::new(32, 32, 48));

    // Slide east
    assert_eq!(
        Collider::new(&s, b + V3::new(1, 0, 0)).calc_velocity(V3::new(10, 10, 0)),
        V3::new(10, 0, 0));

    // Slide south
    assert_eq!(
        Collider::new(&s, b + V3::new(0, 1, 0)).calc_velocity(V3::new(10, 10, 0)),
        V3::new(0, 10, 0));
}

#[test]
fn calc_velocity_ramp_up() {
    // Regression test for a bug that caused boundary checks to happen at min+max instead of
    // min+size.
    let s = init_side_view(&[
        "    __",
        "   /##",
        "__/###",
    ], 2);

    // Position `b` at the left side of the entrance to the ramp.
    let b = Region::sized(V3::new(32, 32, 48)) + V3::new(32, 32 * 4, 0);

    // Trying to walk into the ramp
    assert_eq!(
        Collider::new(&s, b + V3::new(0, 0, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(16, 0, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 0));

    // TODO - NYI
    /*
    // Trying to walk up the edges (blocked)
    assert_eq!(
        Collider::new(&s, b + V3::new(-16, 0, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, 0, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(48, 0, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, 0, 0));
        */

    // Walking up the ramp
    assert_eq!( // ... from the bottom
        Collider::new(&s, b + V3::new(16, -16, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 10));

    assert_eq!( // ... from partway up
        Collider::new(&s, b + V3::new(16, -19, 3)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 10));

    assert_eq!( // ... from dead center on the bottom square
        Collider::new(&s, b + V3::new(16, -32, 16)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 10));

    assert_eq!( // ... from dead center on the top square (toward upper level)
        Collider::new(&s, b + V3::new(16, -64, 48)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 10));

    assert_eq!( // ... onto the top
        Collider::new(&s, b + V3::new(16, -80, 64)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, -10, 0));
}

#[test]
fn calc_velocity_ramp_down() {
    // Regression test for a bug that caused boundary checks to happen at min+max instead of
    // min+size.
    let s = init_side_view(&[
        "    __",
        "   /##",
        "__/###",
    ], 2);

    // Position `b` at the left side of the entrance to the ramp.
    let b = Region::sized(V3::new(32, 32, 48)) + V3::new(32, 32 * 4, 0);

    // Trying to walk into the ramp
    assert_eq!(
        Collider::new(&s, b + V3::new(0, -96, 64)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(16, -96, 64)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, 0));

    // TODO - NYI
    // TODO - need more floor at top side
    /*
    // Trying to walk down the edges (blocked)
    assert_eq!(
        Collider::new(&s, b + V3::new(-16, 0, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, 0, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(48, 0, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, 0, 0));
        */

    // Walking down the ramp
    assert_eq!( // ... from the top
        Collider::new(&s, b + V3::new(16, -80, 64)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, -10));

    assert_eq!( // ... from partway down
        Collider::new(&s, b + V3::new(16, -77, 61)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, -10));

    assert_eq!( // ... from dead center on the top square
        Collider::new(&s, b + V3::new(16, -64, 48)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, -10));

    assert_eq!( // ... from dead center on the bottom square
        Collider::new(&s, b + V3::new(16, -32, 16)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, -10));

    assert_eq!( // ... onto the bottom
        Collider::new(&s, b + V3::new(16, -16, 0)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 10, 0));
}


#[test]
fn walk_basics() {
    let s = init_map(&[
        ".....",
        ".#...",
        ".....",
        ". ...",
        ".....",
    ]);

    let b = Region::sized(V3::new(32, 32, 48));

    let size = 32 * 4;

    // Walking through level ground
    assert_eq!(
        Collider::new(&s, b).walk(V3::new(size, 0, 0), size),
        (V3::new(size, 0, 0), size));

    assert_eq!(
        Collider::new(&s, b).walk(V3::new(0, size, 0), size),
        (V3::new(0, size, 0), size));

    // Walking too far
    assert_eq!(
        Collider::new(&s, b).walk(V3::new(size + 1, 0, 0), size + 1),
        (V3::new(size, 0, 0), size));

    assert_eq!(
        Collider::new(&s, b).walk(V3::new(0, size + 1, 0), size + 1),
        (V3::new(0, size, 0), size));

    // Walking into an object
    assert_eq!(
        Collider::new(&s, b).walk(V3::new(1, 1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(1, 0, 0)).walk(V3::new(1, 1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(0, 1, 0)).walk(V3::new(1, 1, 0), 1),
        (V3::new(0, 0, 0), 0));


    // Walking into an object from further away
    assert_eq!(
        Collider::new(&s, b + V3::new(size, 32, 0)).walk(V3::new(-500, 0, 0), 500),
        (V3::new(-32 * 2, 0, 0), 32 * 2));
}

#[test]
fn walk_edges() {
    let s = init_map(&[
        "......",
        "......",
        "..##..",
        "..##..",
        "......",
        "......",
    ]);

    let b = Region::sized(V3::new(32, 32, 48));

    let mid = 32 * 2 + 16;
    let far = 32 * 5;

    let edge0 = 32 * 1 + 16;
    let edge1 = 32 * 3 + 16;

    // Walking from north
    assert_eq!(
        Collider::new(&s, b + V3::new(mid, 0, 0)).walk(V3::new(0, 500, 0), 500),
        (V3::new(0, 32, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(edge0, 0, 0)).walk(V3::new(0, 500, 0), 500),
        (V3::new(0, 32, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(edge1, 0, 0)).walk(V3::new(0, 500, 0), 500),
        (V3::new(0, 32, 0), 32));

    // Walking from south
    assert_eq!(
        Collider::new(&s, b + V3::new(mid, far, 0)).walk(V3::new(0, -500, 0), 500),
        (V3::new(0, -32, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(edge0, far, 0)).walk(V3::new(0, -500, 0), 500),
        (V3::new(0, -32, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(edge1, far, 0)).walk(V3::new(0, -500, 0), 500),
        (V3::new(0, -32, 0), 32));

    // Walking from west
    assert_eq!(
        Collider::new(&s, b + V3::new(0, mid, 0)).walk(V3::new(500, 0, 0), 500),
        (V3::new(32, 0, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(0, edge0, 0)).walk(V3::new(500, 0, 0), 500),
        (V3::new(32, 0, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(0, edge1, 0)).walk(V3::new(500, 0, 0), 500),
        (V3::new(32, 0, 0), 32));

    // Walking from east
    assert_eq!(
        Collider::new(&s, b + V3::new(far, mid, 0)).walk(V3::new(-500, 0, 0), 500),
        (V3::new(-32, 0, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(far, edge0, 0)).walk(V3::new(-500, 0, 0), 500),
        (V3::new(-32, 0, 0), 32));

    assert_eq!(
        Collider::new(&s, b + V3::new(far, edge1, 0)).walk(V3::new(-500, 0, 0), 500),
        (V3::new(-32, 0, 0), 32));
}

#[test]
fn walk_sliding() {
    let s = init_map(&[
        ".....",
        ".#.#.",
        ".....",
        ".#.#.",
        ".....",
    ]);

    let b = |x,y| Region::sized(V3::new(32, 32, 48)) + V3::new(x, y, 0);

    let run = |x, y, dx, dy| {
        let mut c = Collider::new(&s, b(x, y));
        let v = c.calc_velocity(V3::new(dx, dy, 0));
        c.walk(v, V2::new(dx, dy).max())
    };

    // Walking through level ground
    assert_eq!(run(0, 1,  100, 100), (V3::new(0, 63, 0), 63));          // Y_POS
    assert_eq!(run(128, 1,  -100, 100), (V3::new(0, 63, 0), 63));       // Y_NEG
    assert_eq!(run(1, 0,  100, 100), (V3::new(63, 0, 0), 63));          // Y_POS
    assert_eq!(run(1, 128,  100, -100), (V3::new(63, 0, 0), 63));       // Y_NEG
}



#[test]
fn collide_far() {
    // Regression test for a bug that caused boundary checks to happen at min+max instead of
    // min+size.
    let s = init_map(&[
        ".........",
        ".........",
        ".........",
        "...#.....",
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
    ]);

    let b = Region::sized(V3::new(32, 32, 48));

    let near = 32 * 2;
    let mid = 32 * 3;
    let far = 32 * 4;

    // Velocity check against an object far from the origin
    assert_eq!(
        Collider::new(&s, b + V3::new(mid, near, 0)).calc_velocity(V3::new(0, 10, 0)),
        V3::new(0, 0, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(mid, far, 0)).calc_velocity(V3::new(0, -10, 0)),
        V3::new(0, 0, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(near, mid, 0)).calc_velocity(V3::new(10, 0, 0)),
        V3::new(0, 0, 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(far, mid, 0)).calc_velocity(V3::new(-10, 0, 0)),
        V3::new(0, 0, 0));

    // Walking into an object
    assert_eq!(
        Collider::new(&s, b + V3::new(near, near, 0)).walk(V3::new(1, 1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(near + 1, near, 0)).walk(V3::new(1, 1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(near, near + 1, 0)).walk(V3::new(1, 1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(far, far, 0)).walk(V3::new(-1, -1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(far - 1, far, 0)).walk(V3::new(-1, -1, 0), 1),
        (V3::new(0, 0, 0), 0));

    assert_eq!(
        Collider::new(&s, b + V3::new(far, far - 1, 0)).walk(V3::new(-1, -1, 0), 1),
        (V3::new(0, 0, 0), 0));
}


fn main() {}
