pub trait GlContext {
    type Buffer: GlBuffer;

    fn create_buffer(&mut self) -> Self::Buffer;
}

pub enum BufferTarget {
    Array,
    Index,
}

pub trait GlBuffer {
    fn bind(&mut self, target: BufferTarget);
    fn alloc(&mut self, len: usize);
    fn load(&mut self, offset: usize, data: &[u8]);
    fn len(&self) -> usize;
}
