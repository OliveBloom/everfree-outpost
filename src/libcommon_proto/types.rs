use common_util::Bytes;
use physics::v3::V3;


#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalPos {
    x: u16,
    y: u16,
    z: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalOffset {
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalTime(u16);


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

    pub fn to_global(self, base: V3) -> V3 {
        V3 {
            x: base.x as i32,
            y: base.y as i32,
            z: base.z as i32,
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
