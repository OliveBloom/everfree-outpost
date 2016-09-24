pub mod renderer;

pub mod types;
pub mod structure;
pub mod terrain;
pub mod light;
pub mod entity;


const ATLAS_SIZE: u16 = 32;


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

pub fn emit_quad2<F, V>(mut emit: F, vertex: V)
        where F: FnMut(V), V: Clone+IntrusiveCorner, {
    for &corner in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
        let mut v = vertex.clone();
        *v.corner_mut() = corner;
        emit(v);
    }
}

pub fn remaining_quads<T>(buf: &[T], idx: usize) -> usize {
    (buf.len() - idx) / 6
}


trait GeometryGenerator {
    type Vertex;
    fn generate(&mut self, buf: &mut [Self::Vertex]) -> (usize, bool);
}

trait GeometryGenerator2: Clone {
    type Vertex;

    fn generate<F: FnMut(Self::Vertex)>(&mut self, emit: F);

    fn count_verts(&self) -> usize {
        let mut gen = self.clone();
        let mut count = 0;
        gen.generate(|_| { count += 1; });
        count
    }
}
