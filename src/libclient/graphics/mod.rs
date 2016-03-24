

pub mod renderer;

pub mod types;
pub mod structure;
pub mod terrain;
pub mod lights;


const ATLAS_SIZE: u16 = 32;

const LOCAL_BITS: usize = 3;
const LOCAL_SIZE: i32 = 1 << LOCAL_BITS;


pub trait IntrusiveCorner {
    fn corner(&self) -> &(u8, u8);
    fn corner_mut(&mut self) -> &mut (u8, u8);
}

pub fn emit_quad<T: Copy+IntrusiveCorner>(buf: &mut [T],
                                          idx: &mut usize,
                                          vertex: T) {
    for &corner in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
        buf[*idx] = vertex;
        *buf[*idx].corner_mut() = corner;
        *idx += 1;
    }
}

pub fn remaining_quads<T>(buf: &[T], idx: usize) -> usize {
    (buf.len() - idx) / 6
}


trait GeometryGenerator {
    type Vertex;
    fn generate(&mut self, buf: &mut [Self::Vertex]) -> (usize, bool);
}
