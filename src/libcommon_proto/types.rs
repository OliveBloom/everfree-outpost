use common_util::Bytes;
use physics::v3::V3;


#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalPos {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalOffset {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalTime(pub u16);


impl LocalPos {
    pub fn from_global(pos: V3) -> LocalPos {
        LocalPos {
            x: pos.x as u16,
            y: pos.y as u16,
            z: pos.z as u16,
        }
    }

    pub fn to_global(self, base: V3) -> V3 {
        let dx = self.x.wrapping_sub(base.x as u16);
        let dy = self.y.wrapping_sub(base.y as u16);
        let dz = self.z.wrapping_sub(base.z as u16);
        V3 {
            x: base.x + dx as i16 as i32,
            y: base.y + dy as i16 as i32,
            z: base.z + dz as i16 as i32,
        }
    }

    pub fn from_global_bits(pos: V3, bits: usize) -> LocalPos {
        assert!(bits < 16);
        let mask = (1 << bits) - 1;
        LocalPos {
            x: pos.x as u16 & mask,
            y: pos.y as u16 & mask,
            z: pos.z as u16 & mask,
        }
    }

    pub fn to_global_bits(self, base: V3, bits: usize) -> V3 {
        assert!(bits < 16);
        let mask = (1 << bits) - 1;
        let dx = self.x.wrapping_sub(base.x as u16) & mask;
        let dy = self.y.wrapping_sub(base.y as u16) & mask;
        let dz = self.z.wrapping_sub(base.z as u16) & mask;

        // Sign-extend from `b` bits.  Use a macro to capture `bits` with guaranteed inlining.
        macro_rules! adj {
            ($x:expr) => {
                {
                    let x = $x as i16;
                    // Take the sign bit and shift it over by one.  In a `b`-bit signed integer,
                    // the top bit has value `-(1 << (b - 1))` rather than `1 << (b - 1)`.  We
                    // subtract the difference, `(1 << b)`, to convert unsigned to signed.
                    let adj = (x << 1) & (1 << bits);
                    x - adj
                }
            };
        }

        V3 {
            x: base.x + adj!(dx) as i32,
            y: base.y + adj!(dy) as i32,
            z: base.z + adj!(dz) as i32,
        }
    }

    pub fn unwrap(self) -> V3 {
        V3 {
            x: self.x as i32,
            y: self.y as i32,
            z: self.z as i32,
        }
    }
}

impl LocalOffset {
    pub fn from_global(pos: V3) -> LocalOffset {
        LocalOffset {
            x: pos.x as i16,
            y: pos.y as i16,
            z: pos.z as i16,
        }
    }

    pub fn to_global(self) -> V3 {
        V3 {
            x: self.x as i32,
            y: self.y as i32,
            z: self.z as i32,
        }
    }

    pub fn unwrap(self) -> V3 {
        V3 {
            x: self.x as i32,
            y: self.y as i32,
            z: self.z as i32,
        }
    }
}

impl LocalTime {
    pub fn from_global_32(time: i32) -> LocalTime {
        LocalTime(time as u16)
    }

    pub fn to_global_32(self, base: i32) -> i32 {
        let delta = self.0.wrapping_sub(base as u16);
        base + delta as i16 as i32
    }

    pub fn from_global_64(time: i64) -> LocalTime {
        LocalTime(time as u16)
    }

    pub fn to_global_64(self, base: i64) -> i64 {
        let delta = self.0.wrapping_sub(base as u16);
        base + delta as i16 as i64
    }

    pub fn unwrap(self) -> u16 {
        self.0
    }
}


unsafe impl Bytes for LocalPos {}
unsafe impl Bytes for LocalOffset {}
unsafe impl Bytes for LocalTime {}
