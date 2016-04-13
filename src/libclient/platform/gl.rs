use std::ops::Range;

pub trait Context {
    fn havoc(&mut self);

    type Buffer: Buffer;
    fn create_buffer(&mut self) -> Self::Buffer;
    fn create_buffer_with_data(&mut self, data: &[u8]) -> Self::Buffer {
        let mut b = self.create_buffer();
        b.alloc(data.len());
        b.load(0, data);
        b
    }

    type Shader: Shader;
    fn load_shader(&mut self,
                   vert_name: &str,
                   frag_name: &str,
                   uniforms: &[UniformSpec],
                   arrays: &[ArraySpec],
                   textures: &[TextureSpec],
                   outputs: OutputSpec) -> Self::Shader;

    type Texture: Texture;
    fn load_texture(&mut self, img_name: &str) -> Self::Texture;
    fn texture_import_HACK(&mut self, name: u32, size: (u16, u16)) -> Self::Texture;

    // Would be nice to put this method on Shader, but it needs 
    fn draw(&mut Self::Shader, args: &DrawArgs<Self>);
}

pub struct DrawArgs<'a, GL: ?Sized+Context+'a> {
    pub uniforms: &'a [UniformValue<'a>],
    pub arrays: &'a [&'a GL::Buffer],
    pub textures: &'a [&'a GL::Texture],
    // TODO: output db
    pub index_array: Option<&'a GL::Buffer>,
    pub start: usize,
    pub count: usize,
    // TODO: depth test
    // TODO: blend mode
}

impl<'a, GL: Context> DrawArgs<'a, GL> {
    pub fn new() -> DrawArgs<'a, GL> {
        DrawArgs {
            uniforms: &[],
            arrays: &[],
            textures: &[],
            index_array: None,
            start: 0,
            count: 0,
        }
    }

    pub fn uniforms(&mut self, uniforms: &'a [UniformValue<'a>]) -> &mut Self {
        self.uniforms = uniforms;
        self
    }

    pub fn arrays(&mut self, arrays: &'a [&'a GL::Buffer]) -> &mut Self {
        self.arrays = arrays;
        self
    }

    pub fn textures(&mut self, textures: &'a [&'a GL::Texture]) -> &mut Self {
        self.textures = textures;
        self
    }

    pub fn index_array(&mut self, buffer: &'a GL::Buffer) -> &mut Self {
        self.index_array = Some(buffer);
        self
    }

    pub fn range(&mut self, range: Range<usize>) -> &mut Self {
        assert!(range.end >= range.start);
        self.start = range.start;
        self.count = range.end - range.start;
        self
    }

    pub fn draw(&mut self, shader: &mut GL::Shader) {
        GL::draw(shader, self);
    }
}


pub enum UniformValue<'a> {
    Float(f32),
    V2(&'a [f32; 2]),
    V3(&'a [f32; 3]),
    V4(&'a [f32; 4]),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UniformType {
    Float,
    V2,
    V3,
    V4,
}

pub struct UniformSpec<'a> {
    pub name: &'a str,
    pub ty: UniformType,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataType {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    I8 = 3,
    I16 = 4,
    I32 = 5,
}

pub struct AttribSpec<'a> {
    pub name: &'a str,
    pub offset: u8,
    pub ty: DataType,
    pub normalize: bool,
    pub len: u8,
}

pub struct ArraySpec<'a> {
    pub size: usize,
    pub attribs: &'a [AttribSpec<'a>],
}


pub struct TextureSpec<'a> {
    pub name: &'a str,
}


pub struct OutputSpec {
    pub color_planes: u8,
    pub depth_plane: bool,
}


pub trait Buffer {
    fn alloc(&mut self, len: usize);
    fn load(&mut self, offset: usize, data: &[u8]);
    fn len(&self) -> usize;
}


pub trait Shader {
    fn uniforms_len(&self) -> usize;
    fn arrays_len(&self) -> usize;
    fn textures_len(&self) -> usize;
}

pub trait Texture {
    fn size(&self) -> (u16, u16);
}

/*
pub trait Framebuffer {
    fn size(&self) -> (u16, u16);

    type ColorTexture: Texture;
    fn color_planes_len(&self) -> usize;
    fn color_plane(&self, index: usize) -> Self::ColorTexture;

    type DepthTexture: Texture;
    fn has_depth_plane(&self) -> bool;
    fn depth_plane(&self) -> Self::DepthTexture;
}

*/


macro_rules! uniform {
    ($name:ident : $ty:ident) => {
        $crate::platform::gl::UniformSpec {
            name: stringify!($name),
            ty: $crate::platform::gl::UniformType::$ty,
        }
    };
}

macro_rules! uniforms {
    ($($name:ident : $ty:ident,)*) => {
        &[ $( uniform!($name: $ty), )* ]
    };
}

macro_rules! attrib {
    ($name:ident : $ty:ident [ $len:expr ] (norm) @ $offset:expr) => {
        $crate::platform::gl::AttribSpec {
            name: stringify!($name),
            offset: $offset,
            ty: $crate::platform::gl::DataType::$ty,
            normalize: true,
            len: $len,
        }
    };
    ($name:ident : $ty:ident [ $len:expr ] @ $offset:expr) => {
        $crate::platform::gl::AttribSpec {
            name: stringify!($name),
            offset: $offset,
            ty: $crate::platform::gl::DataType::$ty,
            normalize: false,
            len: $len,
        }
    };
}

macro_rules! attribs {
    ($($name:ident : $ty:ident [ $len:expr ] $( ($norm:ident) )* @ $offset:expr,)*) => {
        &[
            $( attrib!($name: $ty[$len] $(($norm))* @$offset), )*
        ]
    };
}

macro_rules! array {
    ([$size:expr] $attribs:expr) => {
        $crate::platform::gl::ArraySpec {
            size: $size,
            attribs: $attribs,
        }
    };
}

macro_rules! arrays {
    ($([$size:expr] $attribs:expr,)*) => {
        &[
            $( array!([$size] $attribs), )*
        ]
    };
}

macro_rules! texture {
    ($name:ident) => {
        $crate::platform::gl::TextureSpec {
            name: stringify!($name),
        }
    };
}

macro_rules! textures {
    ($($name:ident,)*) => {
        &[ $( texture!($name), )* ]
    };
}

macro_rules! outputs {
    (color: $color:expr) => {
        $crate::platform::gl::OutputSpec {
            color_planes: $color,
            depth_plane: false,
        }
    };
    (color: $color:expr, depth) => {
        $crate::platform::gl::OutputSpec {
            color_planes: $color,
            depth_plane: true,
        }
    };
}
