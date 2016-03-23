use physics::v3::V3;

pub fn unpack_v3(v: (u8, u8, u8)) -> V3 {
    V3::new(v.0 as i32,
            v.1 as i32,
            v.2 as i32)
}
