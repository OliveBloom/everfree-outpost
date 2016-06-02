use std::prelude::v1::*;
use std::intrinsics;
use physics::v3::{V3, V2, Vn, Region, scalar};

pub fn unpack_v3(v: (u8, u8, u8)) -> V3 {
    V3::new(v.0 as i32,
            v.1 as i32,
            v.2 as i32)
}

pub unsafe fn zeroed_boxed_slice<T>(len: usize) -> Box<[T]> {
    let mut v = Vec::with_capacity(len);
    v.set_len(len);
    intrinsics::write_bytes(v.as_mut_ptr(), 0, v.len());
    v.into_boxed_slice()
}

pub fn contains_wrapped(bounds: Region<V2>, pos: V2, mask: V2) -> bool {
    let delta = pos.zip(bounds.min, |a, b| a.wrapping_sub(b)) & mask;
    let size = bounds.max - bounds.min;

    delta.x < size.x &&
    delta.y < size.y
}

pub fn wrap_base(pos: V3, base: V3) -> V3 {
    ((pos - base) & scalar::<V3>(4095)) + base
}
