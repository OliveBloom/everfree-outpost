use core::cmp::{min, max};
use core::fmt;
use core::ops::{Add, Sub, Mul, Div, Rem, Neg, Shl, Shr, BitAnd, BitOr, BitXor, Not};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub type DirAxis = (Axis, bool);

#[allow(non_snake_case, non_upper_case_globals)]
pub const PosX: DirAxis = (Axis::X, false);
#[allow(non_snake_case, non_upper_case_globals)]
pub const PosY: DirAxis = (Axis::Y, false);
#[allow(non_snake_case, non_upper_case_globals)]
pub const PosZ: DirAxis = (Axis::Z, false);
#[allow(non_snake_case, non_upper_case_globals)]
pub const NegX: DirAxis = (Axis::X, true);
#[allow(non_snake_case, non_upper_case_globals)]
pub const NegY: DirAxis = (Axis::Y, true);
#[allow(non_snake_case, non_upper_case_globals)]
pub const NegZ: DirAxis = (Axis::Z, true);

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(asmjs), derive(Hash))]
pub struct V3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl fmt::Debug for V3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (self.x, self.y, self.z).fmt(f)
    }
}

impl V3 {
    #[inline]
    pub fn new(x: i32, y: i32, z: i32) -> V3 {
        V3 { x: x, y: y, z: z }
    }

    #[inline]
    pub fn with_x(self, val: i32) -> V3 {
        self.with(Axis::X, val)
    }

    #[inline]
    pub fn with_y(self, val: i32) -> V3 {
        self.with(Axis::Y, val)
    }

    #[inline]
    pub fn with_z(self, val: i32) -> V3 {
        self.with(Axis::Z, val)
    }

    #[inline]
    pub fn reduce(self) -> V2 {
        V2::new(self.x, self.y)
    }

    #[inline]
    pub fn cross(self, other: V3) -> V3 {
        V3::new(self.y * other.z - self.z * other.y,
                self.z * other.x - self.x * other.z,
                self.x * other.y - self.y * other.z)
    }
}

impl Vn for V3 {
    type Axis = Axis;

    #[inline]
    fn unfold<T, F: FnMut(Axis, T) -> (i32, T)>(val: T, mut f: F) -> (V3, T) {
        let (x, val) = f(Axis::X, val);
        let (y, val) = f(Axis::Y, val);
        let (z, val) = f(Axis::Z, val);
        (V3::new(x, y, z), val)
    }

    #[inline]
    fn get(self, axis: Axis) -> i32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    #[inline]
    fn fold_axes<T, F: FnMut(Axis, T) -> T>(val: T, mut f: F) -> T {
        let val = f(Axis::X, val);
        let val = f(Axis::Y, val);
        let val = f(Axis::Z, val);
        val
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis2 {
    X,
    Y,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(asmjs), derive(Hash))]
pub struct V2 {
    pub x: i32,
    pub y: i32,
}

impl fmt::Debug for V2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (self.x, self.y).fmt(f)
    }
}

impl V2 {
    #[inline]
    pub fn new(x: i32, y: i32) -> V2 {
        V2 { x: x, y: y }
    }

    #[inline]
    pub fn with_x(self, val: i32) -> V2 {
        self.with(Axis2::X, val)
    }

    #[inline]
    pub fn with_y(self, val: i32) -> V2 {
        self.with(Axis2::Y, val)
    }

    #[inline]
    pub fn extend(self, val: i32) -> V3 {
        V3::new(self.x, self.y, val)
    }

    #[inline]
    pub fn cross(self, other: V2) -> i32 {
        self.x * other.y - self.y * other.x
    }
}

impl Vn for V2 {
    type Axis = Axis2;

    #[inline]
    fn unfold<T, F: FnMut(Axis2, T) -> (i32, T)>(val: T, mut f: F) -> (V2, T) {
        let (x, val) = f(Axis2::X, val);
        let (y, val) = f(Axis2::Y, val);
        (V2::new(x, y), val)
    }

    #[inline]
    fn get(self, axis: Axis2) -> i32 {
        match axis {
            Axis2::X => self.x,
            Axis2::Y => self.y,
        }
    }

    #[inline]
    fn fold_axes<T, F: FnMut(Axis2, T) -> T>(val: T, mut f: F) -> T {
        let val = f(Axis2::X, val);
        let val = f(Axis2::Y, val);
        val
    }
}


pub trait Vn:
    Sized + Copy +
    Add<Self, Output=Self> +
    Sub<Self, Output=Self> +
    Mul<Self, Output=Self> +
    Div<Self, Output=Self> +
    Rem<Self, Output=Self> +
    Neg<Output=Self> +
    Shl<usize, Output=Self> +
    Shr<usize, Output=Self> +
    BitAnd<Self, Output=Self> +
    BitOr<Self, Output=Self> +
    BitXor<Self, Output=Self> +
    Not<Output=Self>
{
    type Axis: Eq+Copy;

    fn unfold<T, F: FnMut(<Self as Vn>::Axis, T) -> (i32, T)>(val: T, mut f: F) -> (Self, T);
    fn get(self, axis: <Self as Vn>::Axis) -> i32;
    fn fold_axes<T, F: FnMut(<Self as Vn>::Axis, T) -> T>(init: T, mut f: F) -> T;

    #[inline]
    fn from_fn<F: FnMut(<Self as Vn>::Axis) -> i32>(mut f: F) -> Self {
        <Self as Vn>::unfold((), |a, ()| (f(a), ())).0
    }

    #[inline]
    fn on_axis(axis: <Self as Vn>::Axis, mag: i32) -> Self {
        <Self as Vn>::from_fn(|a| if a == axis { mag } else { 0 })
    }

    #[inline]
    fn on_dir_axis(dir_axis: (<Self as Vn>::Axis, bool), mag: i32) -> Self {
        let (axis, neg) = dir_axis;
        <Self as Vn>::on_axis(axis, if neg { -mag } else { mag })
    }

    #[inline]
    fn map<F: FnMut(i32) -> i32>(self, mut f: F) -> Self {
        <Self as Vn>::from_fn(|a| f(self.get(a)))
    }

    #[inline]
    fn zip<F: FnMut(i32, i32) -> i32>(self, other: Self, mut f: F) -> Self {
        <Self as Vn>::from_fn(|a| f(self.get(a), other.get(a)))
    }

    #[inline]
    fn zip3<F: FnMut(i32, i32, i32) -> i32>(self, other1: Self, other2: Self, mut f: F) -> Self {
        <Self as Vn>::from_fn(|a| f(self.get(a), other1.get(a), other2.get(a)))
    }

    #[inline]
    fn dot(self, other: Self) -> i32 {
        <Self as Vn>::fold_axes(0, |a, sum| sum + self.get(a) * other.get(a))
    }

    #[inline]
    fn mag2(self) -> i32 {
        self.dot(self)
    }

    #[inline]
    fn get_dir(self, dir_axis: (<Self as Vn>::Axis, bool)) -> i32 {
        let (axis, neg) = dir_axis;
        if neg { -self.get(axis) } else { self.get(axis) }
    }

    #[inline]
    fn get_if_pos(self, dir_axis: (<Self as Vn>::Axis, bool)) -> i32 {
        let (axis, neg) = dir_axis;
        if neg { 0 } else { self.get(axis) }
    }

    #[inline]
    fn only(self, axis: <Self as Vn>::Axis) -> Self {
        <Self as Vn>::on_axis(axis, self.get(axis))
    }

    #[inline]
    fn abs(self) -> Self {
        self.map(|x| x.abs())
    }

    #[inline]
    fn signum(self) -> Self {
        self.map(|x| x.signum())
    }

    #[inline]
    fn is_positive(self) -> Self {
        self.map(|x| (x > 0) as i32)
    }

    #[inline]
    fn is_negative(self) -> Self {
        self.map(|x| (x < 0) as i32)
    }

    #[inline]
    fn is_zero(self) -> Self {
        self.map(|x| (x == 0) as i32)
    }

    #[inline]
    fn choose(self, a: Self, b: Self) -> Self {
        self.zip3(a, b, |x, a, b| if x != 0 { a } else { b })
    }

    #[inline]
    fn clamp(self, low: i32, high: i32) -> Self {
        self.map(|x| max(low, min(high, x)))
    }

    #[inline]
    fn with(self, axis: <Self as Vn>::Axis, mag: i32) -> Self {
        <Self as Vn>::from_fn(|a| if a == axis { mag } else { self.get(a) })
    }

    #[inline(always)]
    fn div_floor(self, other: Self) -> Self {
        self.zip(other, |a, b| div_floor(a, b))
    }

    #[inline]
    fn min(self) -> i32 {
        <Self as Vn>::fold_axes(None, |a, x| {
            match x {
                None => Some(self.get(a)),
                Some(x) => Some(min(x, self.get(a))),
            }
        }).unwrap()
    }

    #[inline]
    fn max(self) -> i32 {
        <Self as Vn>::fold_axes(None, |a, x| {
            match x {
                None => Some(self.get(a)),
                Some(x) => Some(max(x, self.get(a))),
            }
        }).unwrap()
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self.zip(other, |a, b| a + b)
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self.zip(other, |a, b| a - b)
    }

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.zip(other, |a, b| a * b)
    }

    #[inline]
    fn div(self, other: Self) -> Self {
        self.zip(other, |a, b| a / b)
    }

    #[inline]
    fn rem(self, other: Self) -> Self {
        self.zip(other, |a, b| a % b)
    }

    #[inline]
    fn neg(self) -> Self {
        self.map(|x| -x)
    }

    #[inline]
    fn shl(self, amount: usize) -> Self {
        self.map(|x| x << amount)
    }

    #[inline]
    fn shr(self, amount: usize) -> Self {
        self.map(|x| x >> amount)
    }

    #[inline]
    fn bitand(self, other: Self) -> Self {
        self.zip(other, |a, b| a & b)
    }

    #[inline]
    fn bitor(self, other: Self) -> Self {
        self.zip(other, |a, b| a | b)
    }

    #[inline]
    fn bitxor(self, other: Self) -> Self {
        self.zip(other, |a, b| a ^ b)
    }

    #[inline]
    fn not(self) -> Self {
        self.map(|x| !x)
    }
}

#[inline]
fn div_floor(a: i32, b: i32) -> i32 {
    if b < 0 {
        return div_floor(-a, -b);
    }

    // In the common case (dividing by a power-of-two constant), we'd like this to turn into a
    // single right-shift instruction.
    if (b as u32).is_power_of_two() {
        let bits = b.trailing_zeros();
        return a >> bits;
    }

    if a < 0 {
        (a - (b - 1)) / b
    } else {
        a / b
    }
}

#[inline]
pub fn scalar<V: Vn>(w: i32) -> V {
    <V as Vn>::from_fn(|_| w)
}


macro_rules! impl_Vn_binop {
    ($vec:ty, $op:ident, $method:ident) => {
        impl $op<$vec> for $vec {
            type Output = $vec;
            #[inline]
            fn $method(self, other: $vec) -> $vec {
                <$vec as Vn>::$method(self, other)
            }
        }
    };
}

macro_rules! impl_Vn_unop {
    ($vec:ty, $op:ident, $method:ident) => {
        impl $op for $vec {
            type Output = $vec;
            #[inline]
            fn $method(self) -> $vec {
                <$vec as Vn>::$method(self)
            }
        }
    };
}

macro_rules! impl_Vn_shift_op {
    ($vec:ty, $op:ident, $method:ident) => {
        impl $op<usize> for $vec {
            type Output = $vec;
            #[inline]
            fn $method(self, amount: usize) -> $vec {
                <$vec as Vn>::$method(self, amount)
            }
        }
    };
}

macro_rules! impl_Vn_ops {
    ($vec:ty) => {
        impl_Vn_binop!($vec, Add, add);
        impl_Vn_binop!($vec, Sub, sub);
        impl_Vn_binop!($vec, Mul, mul);
        impl_Vn_binop!($vec, Div, div);
        impl_Vn_binop!($vec, Rem, rem);
        impl_Vn_unop!($vec, Neg, neg);

        impl_Vn_shift_op!($vec, Shl, shl);
        impl_Vn_shift_op!($vec, Shr, shr);

        impl_Vn_binop!($vec, BitAnd, bitand);
        impl_Vn_binop!($vec, BitOr, bitor);
        impl_Vn_binop!($vec, BitXor, bitxor);
        impl_Vn_unop!($vec, Not, not);
    };
}


impl_Vn_ops!(V3);
impl_Vn_ops!(V2);


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Region<V=V3> {
    pub min: V,
    pub max: V,
}

pub type Region2 = Region<V2>;

impl<V: Copy+fmt::Debug> fmt::Debug for Region<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (self.min, self.max).fmt(f)
    }
}

impl<V: Vn> Region<V> {
    // Constructors

    #[inline]
    pub fn new(min: V, max: V) -> Region<V> {
        Region { min: min, max: max }
    }

    #[inline]
    pub fn empty() -> Region<V> {
        Region { min: scalar(0), max: scalar(0) }
    }

    #[inline]
    pub fn around(center: V, radius: i32) -> Region<V> {
        Region::new(center - scalar(radius),
                    center + scalar(radius))
    }

    #[inline]
    pub fn sized(size: V) -> Region<V> {
        Region::new(scalar(0), size)
    }

    // Basic inspection

    #[inline]
    pub fn is_empty(&self) -> bool {
        <V as Vn>::fold_axes(false, |a, e| e || self.min.get(a) >= self.max.get(a))
    }

    #[inline]
    pub fn size(&self) -> V {
        self.max - self.min
    }

    #[inline]
    pub fn volume(&self) -> i32 {
        let size = self.size();
        <V as Vn>::fold_axes(1, |a, v| v * size.get(a))
    }

    // Region ops

    #[inline]
    pub fn join(&self, other: Region<V>) -> Region<V> {
        Region::new(self.min.zip(other.min, |a, b| min(a, b)),
                    self.max.zip(other.max, |a, b| max(a, b)))
    }

    #[inline]
    pub fn intersect(&self, other: Region<V>) -> Region<V> {
        Region::new(self.min.zip(other.min, |a, b| max(a, b)),
                    self.max.zip(other.max, |a, b| min(a, b)))
    }

    #[inline]
    pub fn overlaps(&self, other: Region<V>) -> bool {
        let inter = self.intersect(other);
        <V as Vn>::fold_axes(true, |a, over| over && inter.min.get(a) < inter.max.get(a))
    }

    #[inline]
    pub fn overlaps_inclusive(&self, other: Region<V>) -> bool {
        let inter = self.intersect(other);
        <V as Vn>::fold_axes(true, |a, over| over && inter.min.get(a) <= inter.max.get(a))
    }

    #[inline]
    pub fn expand(&self, amount: V) -> Region<V> {
        Region::new(self.min - amount, self.max + amount)
    }

    #[inline]
    pub fn div_round(&self, rhs: i32) -> Region<V> {
        Region::new(self.min / scalar(rhs),
                    (self.max + scalar(rhs - 1)) / scalar(rhs))
    }

    #[inline]
    pub fn div_round_signed(&self, rhs: i32) -> Region<V> {
        Region::new(self.min.div_floor(scalar(rhs)),
                    (self.max + scalar(rhs - 1)).div_floor(scalar(rhs)))
    }

    // Point ops

    #[inline]
    pub fn index(&self, point: V) -> usize {
        let offset = point - self.min;
        let size = self.size();
        <V as Vn>::fold_axes((0, 1), |a, (sum, mul)| {
            (sum + offset.get(a) as usize * mul,
             mul * size.get(a) as usize)
        }).0
    }

    #[inline]
    pub fn from_index(&self, idx: usize) -> V {
        let size = self.size();
        <V as Vn>::unfold(idx, |a, acc| {
            let len = size.get(a) as usize;
            let x = self.min.get(a) + (acc % len) as i32;
            let acc = acc / len;
            (x, acc)
        }).0
    }

    #[inline]
    pub fn contains(&self, point: V) -> bool {
        <V as Vn>::fold_axes(true, |a, cur| {
            cur &&
            point.get(a) >= self.min.get(a) &&
            point.get(a) <  self.max.get(a)
        })
    }

    #[inline]
    pub fn contains_inclusive(&self, point: V) -> bool {
        <V as Vn>::fold_axes(true, |a, cur| {
            cur &&
            point.get(a) >= self.min.get(a) &&
            point.get(a) <= self.max.get(a)
        })
    }

    #[inline]
    pub fn clamp_point(&self, point: V) -> V {
        <V as Vn>::from_fn(|a| max(self.min.get(a),
                               min(self.max.get(a) - 1,
                                   point.get(a))))
    }

    // Iteration

    #[inline]
    pub fn points(&self) -> RegionPoints<V> {
        if self.is_empty() {
            RegionPoints::empty()
        } else {
            RegionPoints::new(self.min, self.max)
        }
    }

    #[inline]
    pub fn points_inclusive(&self) -> RegionPoints<V> {
        Region::new(self.min, self.max.map(|x| x + 1)).points()
    }
}

impl Region<V3> {
    #[inline]
    pub fn with_zs(&self, min_z: i32, max_z: i32) -> Region<V3> {
        Region::new(self.min.with_z(min_z), self.max.with_z(max_z))
    }

    #[inline]
    pub fn flatten(&self, depth: i32) -> Region<V3> {
        self.with_zs(self.min.z, self.min.z + depth)
    }

    #[inline]
    pub fn reduce(&self) -> Region<V2> {
        Region::new(self.min.reduce(),
                    self.max.reduce())
    }
}

impl Region<V2> {
    #[inline]
    pub fn extend(&self, min: i32, max: i32) -> Region<V3> {
        Region::new(self.min.extend(min),
                    self.max.extend(max))
    }

    #[inline]
    pub fn align_x(&self, other: Region<V2>, align: Align) -> Region<V2> {
        self.align(other, align, Align::None)
    }

    #[inline]
    pub fn align_y(&self, other: Region<V2>, align: Align) -> Region<V2> {
        self.align(other, Align::None, align)
    }

    #[inline]
    pub fn align(&self, other: Region<V2>, align_x: Align, align_y: Align) -> Region<V2> {
        let delta_x = align_delta(align_x,
                                  self.min.x, self.max.x,
                                  other.min.x, other.max.x);
        let delta_y = align_delta(align_y,
                                  self.min.y, self.max.y,
                                  other.min.y, other.max.y);
        *self + V2::new(delta_x, delta_y)
    }

    /// Compute an inner rect from left/right/top/bottom inset amounts.  A negative inset value
    /// means to inset from the opposite edge instead.  For example:
    ///  * `left = 10`: Left edge of the new rect is 10 units right of the left edge of `self`
    ///  * `left = -10`: Left edge of the new rect is 10 units *left* of the *right* edge of `self`
    #[inline]
    pub fn inset(&self, x0: i32, x1: i32, y0: i32, y1: i32) -> Region<V2> {
        let new_min =
            V2::new(if x0 >= 0 { self.min.x + x0 } else { self.max.x - (-x0) },
                    if y0 >= 0 { self.min.y + y0 } else { self.max.y - (-y0) });
        let new_max =
            V2::new(if x1 >= 0 { self.max.x - x1 } else { self.min.x + (-x1) },
                    if y1 >= 0 { self.max.y - y1 } else { self.min.y + (-y1) });
        Region::new(new_min, new_max)
    }
}

pub enum Align {
    Start,
    End,
    Center,
    None,
}

fn align_delta(align: Align, min1: i32, max1: i32, min2: i32, max2: i32) -> i32 {
    match align {
        Align::Start => min2 - min1,
        Align::End => max2 - max1,
        Align::Center => ((min2 + max2) - (min1 + max1)) >> 1,
        Align::None => 0,
    }
}

macro_rules! impl_Region_binop {
    ($op:ident, $method:ident) => {
        impl<V: Vn+Copy> $op<V> for Region<V> {
            type Output = Region<V>;
            #[inline]
            fn $method(self, other: V) -> Region<V> {
                Region::new(<V as Vn>::$method(self.min, other),
                            <V as Vn>::$method(self.max, other))
            }
        }
    };
}

macro_rules! impl_Region_unop {
    ($op:ident, $method:ident) => {
        impl<V: Vn+Copy> $op for Region<V> {
            type Output = Region<V>;
            #[inline]
            fn $method(self) -> Region<V> {
                Region::new(<V as Vn>::$method(self.min),
                            <V as Vn>::$method(self.max))
            }
        }
    };
}

macro_rules! impl_Region_shift_op {
    ($op:ident, $method:ident) => {
        impl<V: Vn+Copy> $op<usize> for Region<V> {
            type Output = Region<V>;
            #[inline]
            fn $method(self, amount: usize) -> Region<V> {
                Region::new(<V as Vn>::$method(self.min, amount),
                            <V as Vn>::$method(self.max, amount))
            }
        }
    };
}

macro_rules! impl_Region_ops {
    () => {
        impl_Region_binop!(Add, add);
        impl_Region_binop!(Sub, sub);
        impl_Region_binop!(Mul, mul);
        impl_Region_binop!(Div, div);
        impl_Region_binop!(Rem, rem);
        impl_Region_unop!(Neg, neg);

        impl_Region_shift_op!(Shl, shl);
        impl_Region_shift_op!(Shr, shr);

        impl_Region_binop!(BitAnd, bitand);
        impl_Region_binop!(BitOr, bitor);
        impl_Region_binop!(BitXor, bitxor);
        impl_Region_unop!(Not, not);
    };
}

impl_Region_ops!();


#[derive(Clone, Copy)]
pub struct RegionPoints<V> {
    cur: V,
    min: V,
    max: V,
}

impl<V: Vn> RegionPoints<V> {
    #[inline]
    pub fn empty() -> RegionPoints<V> {
        RegionPoints {
            cur: scalar(0),
            min: scalar(0),
            max: scalar(0),
        }
    }

    #[inline]
    pub fn new(min: V, max: V) -> RegionPoints<V> {
        let mut first = true;
        let start = min.map(|x| {
            if first {
                first = false;
                x - 1
            } else {
                x
            }
        });
        RegionPoints {
            cur: start,
            min: min,
            max: max,
        }
    }
}

impl<V: Vn+Copy> Iterator for RegionPoints<V> {
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<V> {
        let (new, carry) = <V as Vn>::unfold(true, |a, carry| {
            if !carry {
                (self.cur.get(a), false)
            } else {
                let new_val = self.cur.get(a) + 1;
                if new_val < self.max.get(a) {
                    (new_val, false)
                } else {
                    (self.min.get(a), true)
                }
            }
        });
        self.cur = new;
        if carry {
            None
        } else {
            Some(new)
        }
    }
}
