use core::cmp;

use v3::{self, V3, V2, Vn, Axis, scalar, Region};

use super::{Shape, ShapeSource};
use super::{TILE_SIZE, TILE_MASK, CHUNK_SIZE};


bitflags! {
    flags Flags: u32 {
        /// Entity is sliding along a wall in the +X direction.
        const SLIDE_X_POS =     0x0000_0001,
        /// Entity is sliding along a wall in the -X direction.
        const SLIDE_X_NEG =     0x0000_0002,
        /// Entity is sliding along a wall in the +Y direction.
        const SLIDE_Y_POS =     0x0000_0004,
        /// Entity is sliding along a wall in the -Y direction.
        const SLIDE_Y_NEG =     0x0000_0008,

        const SLIDE_MASK =      SLIDE_X_POS.bits |
                                SLIDE_X_NEG.bits |
                                SLIDE_Y_POS.bits |
                                SLIDE_Y_NEG.bits,
    }
}


// The core idea of the "walk" physics engine is to construct (implicitly) a map of the boundaries
// between grid cells, indicating which cell edges are passable.  This map is two-dimensional,
// calculated from (roughly) the nearby area under the entity.
//
// Example 1: a floor with an obstacle looks like this:
//
//   *  *  *  *  *
//
//   *  *  *--*  *
//         |  |
//   *  *  *--*  *
//
//   *  *  *  *  *
//
// Example 2: a ramp (one block high) looks like this:
//
//   *  *  *  *
//
//   *--*  *--*
//      |  |
//   *  *  *  *
//
//   *  *  *  *
//
// Note the boundaries at the top and bottom of the ramp are both passable.  The boundaries between
// the upper and lower levels and the left/right boundaries of the ramp are not passable, since
// they do not form a smooth surface.
//
// The boundary check can optionally consider floating obstacles, in addition to discontinuities in
// the floor surface.

/// Main boundary passability function.  `bounds` should describe a border between grid cells,
/// having size zero along `axis`.  Returns true if the boundary described by `bounds` is
/// continuous.
fn check_boundary<S: ShapeSource>(s: &S,
                                  bounds: Region<V3>,
                                  (axis, dir): (Axis, bool)) -> bool {
    assert!(bounds.min.get(axis) % TILE_SIZE == 0,
            "bounds = {:?} does not lie on a cell boundary", bounds);
    assert!(bounds.size().get(axis) == 0);
    let old_bounds = bounds;

    let bounds = Region::new(bounds.min - V3::on_axis(axis, 1),
                             bounds.max).div_round_signed(TILE_SIZE);
    // Like `flatten(1)`, but at the top of the volume instead of the bottom.  This matters when
    // moving up ramps.  If the entity is standing directly on a ramp block, bounds.min.z will be
    // half the ramp height (16 for full-height ramps), which is not high enough to catch the
    // ramp or floor connected to the upper edge of the current ramp (get_shape_below will start
    // with z too low).
    // NB: this only works if the entity's height is > 16.
    let bounds = Region::new(bounds.min.with_z(bounds.max.z - 1),
                             bounds.max);

    // Extra vectors for checking perpendicular edges.
    //      | | | |     <- these ones
    //     ---------
    // `bounds` covers the tiles on the `-` side of the boundary in question.  `perp_offset` gives
    // the offset to the region where perpendicular edges should be checked (either 0 or 1 along
    // `axis`).  `perp_sideways` gives the offset to the cell on the other side of one of those
    // perpendicular boundaries (it would be (1, 0) for the example above).
    let perp_offset = V3::on_axis(axis, (!dir) as i32);
    let other_axis = match axis {
        Axis::X => Axis::Y,
        Axis::Y => Axis::X,
        Axis::Z => panic!("expected Axis::X or Axis::Y"),
    };
    let perp_sideways = V3::on_axis(other_axis, 1);

    for p in bounds.points() {
        let (a, za) = s.get_shape_below(p);
        let (b, zb) = s.get_shape_below(p + V3::on_axis(axis, 1));

        if !boundary_match(a, za, b, zb, axis) {
            return false;
        }

        if p != bounds.min {
            let p2 = p + perp_offset;
            let p1 = p2 - perp_sideways;

            let (a, za) = s.get_shape_below(p1);
            let (b, zb) = s.get_shape_below(p2);

            if !boundary_match(a, za, b, zb, other_axis) {
                return false;
            }
        }
    }

    true
}

fn check_boundary_corner<S: ShapeSource>(s: &S,
                                         corner: V3) -> bool {
    let p = corner.div_floor(scalar(TILE_SIZE));

    let (a, za) = s.get_shape_below(p - V3::new(1, 1, 0));
    let (b, zb) = s.get_shape_below(p - V3::new(0, 1, 0));
    let (c, zc) = s.get_shape_below(p - V3::new(1, 0, 0));
    let (d, zd) = s.get_shape_below(p - V3::new(0, 0, 0));

    boundary_match(a, za, b, zb, Axis::X) &&
    boundary_match(c, zc, d, zd, Axis::X) &&
    boundary_match(a, za, c, zc, Axis::Y) &&
    boundary_match(b, zb, d, zd, Axis::Y)
}

fn boundary_match(a: Shape,
                  za: i32,
                  b: Shape,
                  zb: i32,
                  axis: Axis) -> bool {
    match axis {
        Axis::X => match (a, b) {
            (Shape::Floor, Shape::Floor) => za == zb,
            (Shape::RampN, Shape::RampN) => za == zb,
            _ => false,
        },

        Axis::Y => match (a, b) {
            (Shape::Floor, Shape::Floor) => za == zb,
            (Shape::Floor, Shape::RampN) => za == zb + 1,
            (Shape::RampN, Shape::RampN) => za == zb + 1,
            (Shape::RampN, Shape::Floor) => za == zb,
            _ => false,
        },

        Axis::Z => false,
    }
}


fn calc_planar_velocity<S: ShapeSource>(s: &S,
                                        bounds: Region<V3>,
                                        target: V3) -> (V2, Flags) {
    let corner = bounds.min + bounds.size() * target.is_positive();

    let mut vx = target.x;
    let mut vy = target.y;
    let mut flags = Flags::empty();

    if vx != 0 && corner.x % TILE_SIZE == 0 {
        let x_dir = target.x < 0;
        let flat_x = Region::new(bounds.min.with_x(corner.x),
                                 bounds.max.with_x(corner.x));
        if !check_boundary(s, flat_x, (Axis::X, x_dir)) {
            flags.insert(if vx > 0 { SLIDE_X_POS } else { SLIDE_X_NEG });
            vx = 0;
        }
    }

    if vy != 0 && corner.y % TILE_SIZE == 0 {
        let y_dir = target.y < 0;
        let flat_y = Region::new(bounds.min.with_y(corner.y),
                                 bounds.max.with_y(corner.y));
        if !check_boundary(s, flat_y, (Axis::Y, y_dir)) {
            flags.insert(if vy > 0 { SLIDE_Y_POS } else { SLIDE_Y_NEG });
            vy = 0;
        }
    }

    // When moving directly into a corner, make sure the corner is also continuous.
    if vx != 0 && corner.x % TILE_SIZE == 0 &&
       vy != 0 && corner.y % TILE_SIZE == 0 {
        if !check_boundary_corner(s, corner) {
            vx = 0;
            vy = 0;
            flags.remove(SLIDE_MASK);
        }
    }

    (V2::new(vx, vy), flags)
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FloorState {
    /// The entity is somewhere in mid-air, not touching any floor.
    MidAir,
    /// The entity is standing on a ramp.  The V2 is the slope of the ramp.
    OnRamp(V2),
    /// The entity is standing on a Floor, or balanced on top of a Solid object.
    OnFloor,
    /// The entity's center is in mid-air, but other parts of the entity's base are supported by a
    /// floor.
    PartialFloor,
}

fn center_of_base(r: Region<V3>) -> V3 {
    let r = r.flatten(0);
    r.min + r.size().div_floor(scalar(2))
}

fn get_altitude_below<S: ShapeSource>(s: &S, pos: V3) -> (i32, Shape) {
    get_altitude_below_dir(s, pos, scalar(0))
}

fn get_altitude_below_dir<S: ShapeSource>(s: &S, pos: V3, dir: V2) -> (i32, Shape) {
    // Use `dir` to bias the tile selection.
    // If x == 0 and dir.x == 1, use the normal tile and offset.x = 0.
    // If x == 0 and dir.x == -1, use the tile to the left and offset.x = 32.
    // This ensures we get the right Shape when moving on/off a ramp.  (The altitude should be the
    // same in both cases, assuming the floor is continuous.
    let tile_pos = (pos - dir.is_negative().extend(0)).div_floor(scalar(TILE_SIZE));
    let (shape, tile_z) = s.get_shape_below(tile_pos);
    let offset = pos - tile_pos * scalar(TILE_SIZE);
    let alt = altitude_at_offset(shape, offset.reduce()) + tile_z * TILE_SIZE;
    (alt, shape)
}

fn altitude_at_offset(shape: Shape, off: V2) -> i32 {
    match shape {
        // The `Empty` case should only come up when the entire column is empty.
        // Use an outrageous value here that will never come up in practice.  In particular, -1
        // doesn't work because `walk` will actually check an altitude against -1 when hitting the
        // bottom of a ramp.
        Shape::Empty => -100,
        Shape::Floor => 0,
        Shape::Solid => TILE_SIZE,
        Shape::RampN => TILE_SIZE - off.y,
    }
}

fn check_floor<S: ShapeSource>(s: &S,
                               bounds: Region<V3>,
                               dir: V2) -> FloorState {
    let center = center_of_base(bounds);
    let (alt, shape) = get_altitude_below_dir(s, center, dir);
    if alt == center.z {
        match shape {
            Shape::Empty |  // Should be impossible, but just in case...
            Shape::Floor |
            Shape::Solid => return FloorState::OnFloor,
            Shape::RampN => return FloorState::OnRamp(V2::new(0, -1)),
        }
    }

    // Otherwise, the entity is somewhere in mid-air.

    if center.z % TILE_SIZE != 0 {
        // Skip the unnecessary check, there are no floors between cells.
        return FloorState::MidAir;
    }

    for p in bounds.div_round_signed(TILE_SIZE).flatten(1).points() {
        if s.get_shape(p) == Shape::Floor {
            return FloorState::PartialFloor;
        }
    }

    FloorState::MidAir
}


fn calc_velocity(planar: V2,
                 floor: FloorState) -> V3 {
    match floor {
        // Entities in mid-air fall due to gravity.
        //FloorState::MidAir => V3::new(0, 0, -300),
        FloorState::MidAir => V3::new(0, 0, 0),
        FloorState::OnRamp(slope) => planar.extend(slope.dot(planar)),
        FloorState::OnFloor |
        FloorState::PartialFloor => planar.extend(0),
    }
}


fn walk<S: ShapeSource>(s: &S,
                        bounds: Region<V3>,
                        step: V3,
                        duration: i32,
                        flags: Flags) -> (V3, i32) {
    let start_pos = bounds.min;
    let mut pos = bounds.min;
    let size = bounds.size();
    let center_offset = bounds.flatten(0).size().div_floor(scalar(2));
    let corner_offset = bounds.size() * step.is_positive();

    // Fast path: If the entity is not on a ramp, not sliding, and won't cross into any new cells
    // this tick, then allow the movement.
    // TODO: check sliding flags
    /*
    // TODO: currently broken - doesn't handle the case where the starting position is touching a
    // cell boundary.
    if step.z == 0 {
        let corner = pos + corner_offset;
        let start_tile = corner.reduce().div_floor(scalar(TILE_SIZE));
        let end_tile = (corner + step).reduce().div_floor(scalar(TILE_SIZE));
        if start_tile == end_tile {
            return (step, duration);
        }
    }
    */

    // Bresenham-style line drawing.  Increase accumulator each step, and move along the axes that
    // overflowed.
    let mut acc: V3 = scalar(0);
    let inc = step.abs();
    let limit = inc.max();
    let dir = step.signum();
    let mut steps = 0;

    if limit == 0 {
        return (scalar(0), duration);
    }

    for _ in 0 .. limit {
        acc = acc + inc;
        let overflowed = scalar::<V3>(1) - (acc - scalar(limit)).is_negative();
        acc = acc + scalar::<V3>(limit) * overflowed;


        // Check if there is any reason for this move to be prevented.

        let corner = pos + corner_offset;
        let bounds = Region::new(pos, pos + size);

        // This check handles the "standard" reason for stopping, running into a non-walkable cell
        // boundary.  Such collisions can only happen when passing through a cell boundary.
        if overflowed.x != 0 && corner.x & TILE_MASK == 0 {
            let boundary = Region::new(bounds.min.with_x(corner.x),
                                       bounds.max.with_x(corner.x));
            if !check_boundary(s, boundary, (Axis::X, dir.x < 0)) {
                break;
            }
        }

        if overflowed.y != 0 && corner.y & TILE_MASK == 0 {
            let boundary = Region::new(bounds.min.with_y(corner.y),
                                       bounds.max.with_y(corner.y));
            if !check_boundary(s, boundary, (Axis::Y, dir.y < 0)) {
                break;
            }
        }

        if overflowed.x != 0 && corner.x & TILE_MASK == 0 &&
           overflowed.y != 0 && corner.y & TILE_MASK == 0 {
            if !check_boundary_corner(s, corner) {
                break;
            }
        }

        // TODO: handle stopping due to sliding

        if flags.intersects(SLIDE_MASK) {
            let cur_bounds = Region::sized(bounds.size()) + pos;

            if flags.contains(SLIDE_X_POS) {
                let boundary = Region::new(cur_bounds.min.with_x(cur_bounds.max.x),
                                           cur_bounds.max);
                // Note the lack of `!` on this conditional - we stop if the boundary the entity is
                // sliding against ever becomes *not* blocked.
                if check_boundary(s, boundary, v3::PosX) {
                    break;
                }
            }

            if flags.contains(SLIDE_X_NEG) {
                let boundary = Region::new(cur_bounds.min,
                                           cur_bounds.max.with_x(cur_bounds.min.x));
                if check_boundary(s, boundary, v3::NegX) {
                    break;
                }
            }

            if flags.contains(SLIDE_Y_POS) {
                let boundary = Region::new(cur_bounds.min.with_y(cur_bounds.max.y),
                                           cur_bounds.max);
                if check_boundary(s, boundary, v3::PosY) {
                    break;
                }
            }

            if flags.contains(SLIDE_Y_NEG) {
                let boundary = Region::new(cur_bounds.min,
                                           cur_bounds.max.with_y(cur_bounds.min.y));
                if check_boundary(s, boundary, v3::NegY) {
                    break;
                }
            }
        }

        let new_pos = pos + dir * overflowed;

        let (alt, _) = get_altitude_below(s, new_pos + center_offset);
        if new_pos.z != alt {
            break;
        }


        // All is well, so move to the next position.
        pos = pos + dir * overflowed;
        steps += 1;
    }

    (pos - start_pos, duration * steps / limit)
}


pub fn collide<S: ShapeSource>(s: &S,
                               bounds: Region<V3>,
                               target: V3,
                               duration: i32) -> (V3, i32) {
    let (planar, flags) = calc_planar_velocity(s, bounds, target);
    let floor = check_floor(s, bounds, planar);
    let velocity = calc_velocity(planar, floor);

    let step = (velocity * scalar(duration)).div_floor(scalar(1000));
    walk(s, bounds, step, duration, flags)
}


pub struct Collider<'a, S: ShapeSource+'a> {
    s: &'a S,
    bounds: Region<V3>,

    flags: Flags,
}

impl<'a, S: ShapeSource> Collider<'a, S> {
    pub fn new(s: &'a S, bounds: Region<V3>) -> Collider<'a, S> {
        Collider {
            s: s,
            bounds: bounds,

            flags: Flags::empty(),
        }
    }

    pub fn calc_velocity(&mut self, target: V3) -> V3 {
        let (planar, flags) = calc_planar_velocity(self.s, self.bounds, target);
        let floor = check_floor(self.s, self.bounds, planar);
        let velocity = calc_velocity(planar, floor);
        self.flags = flags;
        velocity
    }

    pub fn walk(&self, step: V3, duration: i32) -> (V3, i32) {
        walk(self.s, self.bounds, step, duration, self.flags)
    }
}
