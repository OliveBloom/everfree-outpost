use std::prelude::v1::*;
use std::cell::{Cell, UnsafeCell};
use std::f32;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::str::FromStr;

use client::platform::gl;
use client::platform::gl::{DrawArgs, UniformValue};

mod ffi {
    extern "C" {
        pub fn asmgl_gen_buffer() -> u32;
        pub fn asmgl_delete_buffer(name: u32);
        pub fn asmgl_bind_buffer(target: u8, name: u32);
        pub fn asmgl_buffer_data_alloc(target: u8, len: usize);
        pub fn asmgl_buffer_subdata(target: u8, offset: usize, ptr: *const u8, len: usize);

        pub fn asmgl_load_shader(vert_name_ptr: *const u8,
                                 vert_name_len: usize,
                                 frag_name_ptr: *const u8,
                                 frag_name_len: usize) -> u32;
        pub fn asmgl_delete_shader(name: u32);
        pub fn asmgl_bind_shader(name: u32);
        pub fn asmgl_get_uniform_location(shader_name: u32,
                                          name_ptr: *const u8,
                                          name_len: usize) -> i32;
        pub fn asmgl_get_attrib_location(shader_name: u32,
                                         name_ptr: *const u8,
                                         name_len: usize) -> i32;
        pub fn asmgl_set_uniform_1i(location: i32, value: i32);
        pub fn asmgl_set_uniform_1f(location: i32, value: f32);
        pub fn asmgl_set_uniform_2f(location: i32, value: &[f32; 2]);
        pub fn asmgl_set_uniform_3f(location: i32, value: &[f32; 3]);
        pub fn asmgl_set_uniform_4f(location: i32, value: &[f32; 4]);

        pub fn asmgl_load_texture(name_ptr: *const u8,
                                  name_len: usize,
                                  size_p: *mut (u16, u16)) -> u32;
        pub fn asmgl_delete_texture(name: u32);
        pub fn asmgl_active_texture(unit: usize);
        pub fn asmgl_bind_texture(name: u32);

        pub fn asmgl_enable_vertex_attrib_array(loc: i32);
        pub fn asmgl_disable_vertex_attrib_array(loc: i32);
        pub fn asmgl_vertex_attrib_pointer(loc: i32,
                                           count: usize,
                                           ty: u8,
                                           normalize: u8,
                                           stride: usize,
                                           offset: usize);
        pub fn asmgl_draw_arrays_triangles(start: usize, count: usize);

    }
}


#[unsafe_no_drop_flag]
struct InnerPtr(*mut (UnsafeCell<Inner>, Cell<usize>));

impl InnerPtr {
    fn new() -> InnerPtr {
        let b = box (UnsafeCell::new(Inner::new()), Cell::new(1));
        InnerPtr(Box::into_raw(b))
    }

    fn run<R, F: FnOnce(&mut Inner) -> R>(&mut self, f: F) -> R {
        unsafe {
            let ptr = (*self.0).0.get();
            f(&mut *ptr)
        }
    }
}

impl Drop for InnerPtr {
    fn drop(&mut self) {
        if self.0 as usize == mem::POST_DROP_USIZE {
            return;
        }

        unsafe {
            let ptr = self.0;
            let count = (*ptr).1.get();
            if count > 1 {
                (*ptr).1.set(count - 1);
            } else {
                // This is the last reference
                drop(Box::from_raw(ptr));
            }
            self.0 = mem::POST_DROP_USIZE as *mut _;
        }
    }
}

impl Clone for InnerPtr {
    fn clone(&self) -> InnerPtr {
        unsafe {
            let ptr = self.0;
            let count = (*ptr).1.get();
            (*ptr).1.set(count + 1);
            InnerPtr(ptr)
        }
    }
}


// Typesafe names

pub struct Name<T> {
    pub raw: u32,
    _marker: PhantomData<T>,
}

impl<T> Clone for Name<T> {
    fn clone(&self) -> Name<T> { Name::new(self.raw) }
}
impl<T> Copy for Name<T> {}

impl<T> PartialEq for Name<T> {
    fn eq(&self, other: &Name<T>) -> bool { self.raw == other.raw }
}
impl<T> Eq for Name<T> {}

impl<T> fmt::Debug for Name<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Name({})", self.raw)
    }
}

impl<T> Name<T> {
    pub fn new(raw: u32) -> Name<T> {
        Name {
            raw: raw,
            _marker: PhantomData,
        }
    }
}

const NO_BUFFER: Name<Buffer> = Name { raw: 0, _marker: PhantomData };
const NO_SHADER: Name<Shader> = Name { raw: 0, _marker: PhantomData };
const NO_TEXTURE: Name<Texture> = Name { raw: 0, _marker: PhantomData };


// State tracker internals

#[repr(u8)]
pub enum BufferTarget {
    Array = 0,
    ElementArray = 1,
}


// The spec guarantees a minimum of 8, though implementations may provide more.
pub const NUM_TEXTURE_UNITS: usize = 8;

struct Inner {
    buffers: [Name<Buffer>; 2],
    shader: Name<Shader>,
    texture_unit: usize,
    textures: [Name<Texture>; NUM_TEXTURE_UNITS],
    vertex_attrib_mask: u32,
}

impl Inner {
    fn new() -> Inner {
        Inner {
            buffers: [NO_BUFFER; 2],
            shader: NO_SHADER,
            texture_unit: 0,
            textures: [NO_TEXTURE; NUM_TEXTURE_UNITS],
            vertex_attrib_mask: 0,
        }
    }

    pub fn havoc(&mut self) {
        self.buffers = [NO_BUFFER; 2];
        self.shader = NO_SHADER;
        // NUM_TEXTURE_UNITS is not a valid texture unit index
        self.texture_unit = NUM_TEXTURE_UNITS;
        self.textures = [NO_TEXTURE; NUM_TEXTURE_UNITS];
        // Setting to 0 isn't perfect, but leaving stuff enabled is better than the opposite.
        self.vertex_attrib_mask = 0;
    }

    // Internal API.  This basically wraps OpenGL, but the implementation does its own caching in
    // some places.

    // Buffer objects

    pub fn gen_buffer(&mut self) -> Name<Buffer> {
        let name = unsafe { ffi::asmgl_gen_buffer() };
        assert!(name != 0);
        Name::new(name)
    }

    pub fn delete_buffer(&mut self, name: Name<Buffer>) {
        unsafe { ffi::asmgl_delete_buffer(name.raw) };
        for b in &mut self.buffers {
            if *b == name {
                *b = NO_BUFFER;
            }
        }
    }

    pub fn bind_buffer(&mut self, target: BufferTarget, name: Name<Buffer>) {
        let target_idx = target as u8;
        if self.buffers[target_idx as usize] != name {
            unsafe { ffi::asmgl_bind_buffer(target_idx, name.raw) };
            self.buffers[target_idx as usize] = name;
        }
    }

    pub fn buffer_alloc(&mut self, target: BufferTarget, len: usize) {
        unsafe { ffi::asmgl_buffer_data_alloc(target as u8, len) };
    }

    pub fn buffer_subdata(&mut self,
                          target: BufferTarget,
                          offset: usize,
                          data: &[u8]) {
        unsafe { ffi::asmgl_buffer_subdata(target as u8,
                                           offset,
                                           data.as_ptr(),
                                           data.len()) };
    }

    // Shaders

    pub fn load_shader(&mut self,
                       vert_name: &str,
                       frag_name: &str) -> Name<Shader> {
        let name = unsafe {
            ffi::asmgl_load_shader(vert_name.as_ptr(),
                                   vert_name.len(),
                                   frag_name.as_ptr(),
                                   frag_name.len())
        };
        assert!(name != 0);
        Name::new(name)
    }

    pub fn delete_shader(&mut self, name: Name<Shader>) {
        unsafe { ffi::asmgl_delete_buffer(name.raw) };
        if self.shader == name {
            self.shader = NO_SHADER;
        }
    }

    pub fn bind_shader(&mut self, name: Name<Shader>) {
        if self.shader != name {
            unsafe { ffi::asmgl_bind_shader(name.raw) };
            self.shader = name;
        }
    }

    pub fn uniform_location(&mut self, shader_name: Name<Shader>, var_name: &str) -> i32 {
        unsafe {
            ffi::asmgl_get_uniform_location(shader_name.raw,
                                            var_name.as_ptr(),
                                            var_name.len())
        }
    }

    pub fn attrib_location(&mut self, shader_name: Name<Shader>, var_name: &str) -> i32 {
        unsafe {
            ffi::asmgl_get_attrib_location(shader_name.raw,
                                           var_name.as_ptr(),
                                           var_name.len())
        }
    }

    pub fn set_uniform_1i(&mut self, location: i32, value: i32) {
        unsafe { ffi::asmgl_set_uniform_1i(location, value) }
    }
    pub fn set_uniform_1f(&mut self, location: i32, value: f32) {
        unsafe { ffi::asmgl_set_uniform_1f(location, value) }
    }
    pub fn set_uniform_2f(&mut self, location: i32, value: &[f32; 2]) {
        unsafe { ffi::asmgl_set_uniform_2f(location, value) }
    }
    pub fn set_uniform_3f(&mut self, location: i32, value: &[f32; 3]) {
        unsafe { ffi::asmgl_set_uniform_3f(location, value) }
    }
    pub fn set_uniform_4f(&mut self, location: i32, value: &[f32; 4]) {
        unsafe { ffi::asmgl_set_uniform_4f(location, value) }
    }

    // Textures

    pub fn load_texture(&mut self,
                        img_name: &str,
                        size: &mut (u16, u16)) -> Name<Texture> {
        let name = unsafe {
            ffi::asmgl_load_texture(img_name.as_ptr(),
                                    img_name.len(),
                                    size)
        };
        assert!(name != 0);
        let name = Name::new(name);
        // As a side effect, load_texture also binds the texture to the context, and sets the
        // current texture image unit to 0.
        self.texture_unit = 0;
        self.textures[0] = name;
        name
    }

    pub fn delete_texture(&mut self, name: Name<Texture>) {
        unsafe { ffi::asmgl_delete_texture(name.raw) };
        for t in &mut self.textures {
            if *t == name {
                *t = NO_TEXTURE;
            }
        }
    }

    pub fn bind_texture(&mut self,
                        unit: usize,
                        name: Name<Texture>) {
        if name != self.textures[unit] {
            if unit != self.texture_unit {
                unsafe { ffi::asmgl_active_texture(unit) };
                self.texture_unit = unit;
            }

            unsafe { ffi::asmgl_bind_texture(name.raw) };
            self.textures[unit] = name;
        }
    }


    // Drawing

    pub fn set_vertex_attrib_mask(&mut self, mask: u32) {
        let switch = mask ^ self.vertex_attrib_mask;
        if switch == 0 {
            return;
        }

        for i in 0 .. 32 {
            if (switch & (1 << i)) != 0 {
                if (mask & (1 << i)) != 0 {
                    unsafe { ffi::asmgl_enable_vertex_attrib_array(i) };
                } else {
                    unsafe { ffi::asmgl_disable_vertex_attrib_array(i) };
                }
            }
        }

        self.vertex_attrib_mask = mask;
    }

    pub fn vertex_attrib_buffer(&mut self,
                                loc: i32,
                                count: usize,
                                ty: gl::DataType,
                                normalize: bool,
                                stride: usize,
                                offset: usize) {
        unsafe {
            ffi::asmgl_vertex_attrib_pointer(
                loc, count, ty as u8, normalize as u8, stride, offset);
        }
    }

    pub fn draw_triangles(&mut self, start: usize, count: usize) {
        unsafe {
            ffi::asmgl_draw_arrays_triangles(start, count);
        }
    }
}


// GL

pub struct GL {
    inner: InnerPtr,
}

impl GL {
    pub fn new() -> GL {
        GL { inner: InnerPtr::new() }
    }
}

impl gl::Context for GL {
    fn havoc(&mut self) {
        self.inner.run(|ctx| ctx.havoc());
    }

    type Buffer = Buffer;

    fn create_buffer(&mut self) -> Buffer {
        let name = self.inner.run(|ctx| ctx.gen_buffer());
        Buffer {
            inner: self.inner.clone(),
            len: 0,
            name: name,
        }
    }

    type Shader = Shader;

    fn load_shader(&mut self,
                   vert_name: &str,
                   frag_name: &str,
                   uniforms: &[gl::UniformSpec],
                   arrays: &[gl::ArraySpec],
                   textures: &[gl::TextureSpec],
                   outputs: gl::OutputSpec) -> Shader {
        let inner = self.inner.clone();

        self.inner.run(|ctx| {
            let name = ctx.load_shader(vert_name, frag_name);
            ctx.bind_shader(name);

            let mut uniform_vec = Vec::with_capacity(uniforms.len());
            for u in uniforms {
                let loc = ctx.uniform_location(name, u.name);
                uniform_vec.push(Uniform {
                    location: loc,
                    last_value: [f32::NAN; 4],
                });
            }

            let num_attribs = arrays.iter().map(|a| a.attribs.len()).sum();
            let mut attrib_vec = Vec::with_capacity(num_attribs);
            let mut attrib_mask = 0;
            for (i, arr) in arrays.iter().enumerate() {
                for a in arr.attribs {
                    let loc = ctx.attrib_location(name, a.name);
                    assert!(loc < 32);
                    attrib_mask |= 1 << loc;
                    attrib_vec.push(Attrib {
                        location: loc,
                        array_idx: i as u8,
                        count: a.len,
                        ty: a.ty,
                        normalize: a.normalize,
                        stride: arr.size as u8,
                        offset: a.offset,
                    });
                }
            }

            for (i, t) in textures.iter().enumerate() {
                // Set the sampler for texture `i` to use texture image unit `i`.
                let uniform_loc = ctx.uniform_location(name, t.name);
                ctx.set_uniform_1i(uniform_loc, i as i32);
            }

            Shader {
                inner: inner,
                name: name,

                uniforms: uniform_vec.into_boxed_slice(),
                attribs: attrib_vec.into_boxed_slice(),
                attrib_mask: attrib_mask,
                num_arrays: arrays.len() as u8,
                num_textures: textures.len() as u8,
            }
        })
    }

    type Texture = Texture;

    fn load_texture(&mut self, img_name: &str) -> Texture {
        let mut size = (0, 0);
        let name = self.inner.run(|ctx| ctx.load_texture(img_name, &mut size));
        Texture {
            inner: self.inner.clone(),
            name: name,
            size: size,
        }
    }

    fn texture_import_HACK(&mut self, name: u32, size: (u16, u16)) -> Texture {
        Texture {
            inner: self.inner.clone(),
            name: Name::new(name),
            size: size,
        }
    }


    fn draw(shader: &mut Shader, args: &DrawArgs<GL>) {
        shader.draw(args);
    }
}


pub struct Buffer {
    inner: InnerPtr,
    len: usize,
    name: Name<Buffer>,
}

impl Buffer {
    pub fn name(&self) -> Name<Buffer> {
        self.name
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl gl::Buffer for Buffer {
    fn alloc(&mut self, len: usize) {
        if len == self.len {
            return;
        }

        let name = self.name;
        self.inner.run(|ctx| {
            ctx.bind_buffer(BufferTarget::Array, name);
            ctx.buffer_alloc(BufferTarget::Array, len);
        });
        self.len = len;
    }

    fn load(&mut self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() <= self.len);
        let name = self.name;
        self.inner.run(|ctx| {
            ctx.bind_buffer(BufferTarget::Array, name);
            ctx.buffer_subdata(BufferTarget::Array, offset, data);
        });
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let name = self.name;
        self.inner.run(move |ctx| {
            ctx.delete_buffer(name);
        });
    }
}


struct Uniform {
    location: i32,
    last_value: [f32; 4],
}

struct Attrib {
    location: i32,
    array_idx: u8,
    count: u8, 
    ty: gl::DataType,
    normalize: bool,
    stride: u8,
    offset: u8,
}

pub struct Shader {
    inner: InnerPtr,
    name: Name<Shader>,

    uniforms: Box<[Uniform]>,
    attribs: Box<[Attrib]>,
    attrib_mask: u32,
    num_arrays: u8,
    num_textures: u8,
}

impl Shader {
    fn draw(&mut self, args: &DrawArgs<GL>) {
        let mut inner = self.inner.clone();
        inner.run(|ctx| {
            assert!(args.uniforms.len() == self.uniforms.len());
            assert!(args.arrays.len() == self.num_arrays as usize);
            assert!(args.textures.len() == self.num_textures as usize);

            ctx.bind_shader(self.name);

            for (u, v) in self.uniforms.iter_mut().zip(args.uniforms.iter()) {
                Shader::set_uniform_value(ctx, u, v);
            }

            ctx.set_vertex_attrib_mask(self.attrib_mask);
            for a in self.attribs.iter() {
                let buf_name = args.arrays[a.array_idx as usize].name;
                ctx.bind_buffer(BufferTarget::Array, buf_name);
                ctx.vertex_attrib_buffer(a.location,
                                         a.count as usize,
                                         a.ty,
                                         a.normalize,
                                         a.stride as usize,
                                         a.offset as usize);
            }

            for (i, t) in args.textures.iter().enumerate() {
                ctx.bind_texture(i, t.name);
            }

            ctx.draw_triangles(args.start, args.count);
        });
    }

    fn set_uniform_value(ctx: &mut Inner, u: &mut Uniform, v: &UniformValue) {
        match *v {
            UniformValue::Float(x) => {
                if u.last_value[0] != x {
                    ctx.set_uniform_1f(u.location, x);
                    u.last_value[0] = x;
                }
            },
            UniformValue::V2(x) => {
                if &u.last_value[0..2] != x {
                    ctx.set_uniform_2f(u.location, x);
                    for i in 0 .. 2 {
                        u.last_value[i] = x[i];
                    }
                }
            },
            UniformValue::V3(x) => {
                if &u.last_value[0..3] != x {
                    ctx.set_uniform_3f(u.location, x);
                    for i in 0 .. 3 {
                        u.last_value[i] = x[i];
                    }
                }
            },
            UniformValue::V4(x) => {
                if &u.last_value[0..4] != x {
                    ctx.set_uniform_4f(u.location, x);
                    for i in 0 .. 4 {
                        u.last_value[i] = x[i];
                    }
                }
            },
        }
    }
}

impl gl::Shader for Shader {
    fn uniforms_len(&self) -> usize {
        self.uniforms.len()
    }

    fn arrays_len(&self) -> usize {
        self.num_arrays as usize
    }

    fn textures_len(&self) -> usize {
        self.num_textures as usize
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let name = self.name;
        self.inner.run(move |ctx| {
            ctx.delete_shader(name);
        });
    }
}


pub struct Texture {
    inner: InnerPtr,
    name: Name<Texture>,
    size: (u16, u16),
}

impl gl::Texture for Texture {
    fn size(&self) -> (u16, u16) {
        self.size
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        let name = self.name;
        self.inner.run(move |ctx| {
            ctx.delete_texture(name);
        });
    }
}
