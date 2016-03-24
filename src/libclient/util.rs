use std::prelude::v1::*;
use std::intrinsics;
use physics::v3::V3;

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
