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

pub fn emit_quad<F, V>(mut emit: F, vertex: V)
        where F: FnMut(V), V: Clone+IntrusiveCorner, {
    for &corner in &[(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
        let mut v = vertex.clone();
        *v.corner_mut() = corner;
        emit(v);
    }
}


trait GeometryGenerator: Clone {
    type Vertex;

    fn generate<F: FnMut(Self::Vertex)>(&mut self, emit: F);

    fn count_verts(&self) -> usize {
        let mut gen = self.clone();
        let mut count = 0;
        gen.generate(|_| { count += 1; });
        count
    }
}
