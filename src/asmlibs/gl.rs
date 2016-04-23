use std::prelude::v1::*;
use std::cell::{Cell, UnsafeCell};
use std::f32;
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::slice;
use std::str::FromStr;

use physics::v3::{Region, V2, scalar};

use client::platform::gl;
use client::platform::gl::{DrawArgs, UniformValue};

mod ffi {
    extern "C" {
        pub fn asmgl_has_extension(ptr: *const u8, len: usize) -> u8;

        pub fn asmgl_gen_buffer() -> u32;
        pub fn asmgl_delete_buffer(name: u32);
        pub fn asmgl_bind_buffer(target: u8, name: u32);
        pub fn asmgl_buffer_data_alloc(target: u8, len: usize);
        pub fn asmgl_buffer_subdata(target: u8, offset: usize, ptr: *const u8, len: usize);

        pub fn asmgl_load_shader(vert_name_ptr: *const u8,
                                 vert_name_len: usize,
                                 frag_name_ptr: *const u8,
                                 frag_name_len: usize,
                                 defs_ptr: *const u8,
                                 defs_len: usize) -> u32;
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
        pub fn asmgl_gen_texture(width: u16, height: u16, kind: u8) -> u32;
        pub fn asmgl_delete_texture(name: u32);
        pub fn asmgl_active_texture(unit: usize);
        pub fn asmgl_bind_texture(name: u32);
        pub fn asmgl_texture_image(width: u16,
                                   height: u16,
                                   kind: u8,
                                   data_ptr: *const u8,
                                   data_len: usize);
        pub fn asmgl_texture_subimage(x: u16,
                                      y: u16,
                                      width: u16,
                                      height: u16,
                                      kind: u8,
                                      data_ptr: *const u8,
                                      data_len: usize);

        pub fn asmgl_gen_framebuffer() -> u32;
        pub fn asmgl_delete_framebuffer(name: u32);
        pub fn asmgl_bind_framebuffer(name: u32);
        pub fn asmgl_gen_renderbuffer(width: u16, height: u16, is_depth: u8) -> u32;
        pub fn asmgl_delete_renderbuffer(name: u32);
        // No use for bind_renderbuffer so far
        pub fn asmgl_framebuffer_texture(tex_name: u32,
                                         attachment: i8);
        pub fn asmgl_framebuffer_renderbuffer(rb_name: u32,
                                              attachment: i8);
        pub fn asmgl_check_framebuffer_status() -> u8;
        pub fn asmgl_draw_buffers(num_attachments: u8);

        pub fn asmgl_viewport(x: i32, y: i32, w: i32, h: i32);
        pub fn asmgl_clear_color(r: f32, g: f32, b: f32, a: f32);
        pub fn asmgl_clear_depth(d: f32);
        pub fn asmgl_clear();
        pub fn asmgl_set_depth_test(enabled: u8);
        pub fn asmgl_set_blend_mode(mode: u8);
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

    fn run_imm<R, F: FnOnce(&Inner) -> R>(&self, f: F) -> R {
        unsafe {
            let ptr = (*self.0).0.get();
            f(&*ptr)
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
const NO_FRAMEBUFFER: Name<Framebuffer> = Name { raw: 0, _marker: PhantomData };
const NO_RENDERBUFFER: Name<Renderbuffer> = Name { raw: 0, _marker: PhantomData };


// State tracker internals

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum BufferTarget {
    Array = 0,
    ElementArray = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum TextureKind {
    RGBA = 0,
    Depth = 1,
    Luminance = 2,
}


mod exts {
    #![allow(non_upper_case_globals)]
    bitflags! {
        pub flags Extensions: u32 {
            const WEBGL_depth_texture =         0x00000001,
            const WEBGL_draw_buffers =          0x00000002,
            const OES_vertex_array_object =     0x00000004,
        }
    }
}
pub use self::exts::*;

fn check_extension(name: &str) -> bool {
    unsafe { ffi::asmgl_has_extension(name.as_ptr(), name.len()) != 0 }
}

fn detect_extensions() -> Extensions {
    let mut exts = Extensions::empty();
    macro_rules! check {
        ($name:ident) => {
            if check_extension(stringify!($name)) {
                exts.insert($name);
            }
        };
    };

    check!(WEBGL_depth_texture);
    check!(WEBGL_draw_buffers);
    check!(OES_vertex_array_object);

    println!("detected extensions: {:?}", exts);
    exts
}


// The spec guarantees a minimum of 8, though implementations may provide more.
pub const NUM_TEXTURE_UNITS: usize = 8;

struct Inner {
    extensions: Extensions,
    buffers: [Name<Buffer>; 2],
    shader: Name<Shader>,
    texture_unit: usize,
    textures: [Name<Texture>; NUM_TEXTURE_UNITS],
    vertex_attrib_mask: u32,
    framebuffer: Name<Framebuffer>,
    viewport: Region<V2>,
    depth_test_enabled: bool,
    blend_mode: gl::BlendMode,
}

impl Inner {
    fn new() -> Inner {
        Inner {
            extensions: detect_extensions(),
            buffers: [NO_BUFFER; 2],
            shader: NO_SHADER,
            texture_unit: 0,
            textures: [NO_TEXTURE; NUM_TEXTURE_UNITS],
            vertex_attrib_mask: 0,
            framebuffer: NO_FRAMEBUFFER,
            viewport: Region::sized(scalar(0)),
            depth_test_enabled: false,
            blend_mode: gl::BlendMode::None,
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
        self.framebuffer = NO_FRAMEBUFFER;
        self.depth_test_enabled = false;
        self.blend_mode = gl::BlendMode::None;
    }

    // Internal API.  This basically wraps OpenGL, but the implementation does its own caching in
    // some places.

    fn has(&self, ext: Extensions) -> bool {
        self.extensions.contains(ext)
    }

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
                       frag_name: &str,
                       defs: &str) -> Name<Shader> {
        let name = unsafe {
            ffi::asmgl_load_shader(vert_name.as_ptr(),
                                   vert_name.len(),
                                   frag_name.as_ptr(),
                                   frag_name.len(),
                                   defs.as_ptr(),
                                   defs.len())
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

    pub fn gen_texture(&mut self, size: (u16, u16), kind: TextureKind) -> Name<Texture> {
        let name = unsafe { ffi::asmgl_gen_texture(size.0, size.1, kind as u8) };
        assert!(name != 0);
        let name = Name::new(name);
        // As a side effect, gen_texture also binds the texture to the context,
        // and sets the current texture image unit to 0.
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

    pub fn texture_image(&mut self,
                         size: (u16, u16),
                         kind: TextureKind,
                         data: &[u8]) {
        unsafe {
            ffi::asmgl_texture_image(size.0,
                                     size.1,
                                     kind as u8,
                                     data.as_ptr(),
                                     data.len());
        }
    }

    pub fn texture_subimage(&mut self,
                            offset: (u16, u16),
                            size: (u16, u16),
                            kind: TextureKind,
                            data: &[u8]) {
        unsafe {
            ffi::asmgl_texture_subimage(offset.0,
                                        offset.1,
                                        size.0,
                                        size.1,
                                        kind as u8,
                                        data.as_ptr(),
                                        data.len());
        }
    }


    // Framebuffer objects

    pub fn gen_framebuffer(&mut self) -> Name<Framebuffer> {
        let name = unsafe { ffi::asmgl_gen_framebuffer() };
        assert!(name != 0);
        Name::new(name)
    }

    pub fn delete_framebuffer(&mut self, name: Name<Framebuffer>) {
        unsafe { ffi::asmgl_delete_framebuffer(name.raw) };
        if self.framebuffer == name {
            self.framebuffer = NO_FRAMEBUFFER;
        }
    }

    pub fn bind_framebuffer(&mut self, name: Name<Framebuffer>) {
        if name != self.framebuffer {
            unsafe { ffi::asmgl_bind_framebuffer(name.raw) };
            self.framebuffer = name;
        }
    }

    pub fn gen_renderbuffer(&mut self, size: (u16, u16), is_depth: bool) -> Name<Renderbuffer> {
        let name = unsafe { ffi::asmgl_gen_renderbuffer(size.0, size.1, is_depth as u8) };
        assert!(name != 0);
        // This does affect the GL_RENDERBUFFER binding, but we don't actually keep track of that.
        Name::new(name)
    }

    pub fn delete_renderbuffer(&mut self, name: Name<Renderbuffer>) {
        unsafe { ffi::asmgl_delete_renderbuffer(name.raw) };
    }

    pub fn framebuffer_texture(&mut self,
                               tex_name: Name<Texture>,
                               attachment: i8) {
        unsafe { ffi::asmgl_framebuffer_texture(tex_name.raw, attachment) };
    }

    pub fn framebuffer_renderbuffer(&mut self,
                                    rb_name: Name<Renderbuffer>,
                                    attachment: i8) {
        unsafe { ffi::asmgl_framebuffer_renderbuffer(rb_name.raw, attachment) };
    }

    pub fn is_framebuffer_complete(&mut self) -> bool {
        let status = unsafe { ffi::asmgl_check_framebuffer_status() };
        status != 0
    }


    // Drawing

    pub fn viewport(&mut self, bounds: Region<V2>) {
        if self.viewport != bounds {
            unsafe {
                ffi::asmgl_viewport(bounds.min.x,
                                    bounds.min.y,
                                    bounds.size().x,
                                    bounds.size().y);
            }
            self.viewport = bounds;
        }
    }

    pub fn clear_color(&mut self, color: (u8, u8, u8, u8)) {
        unsafe {
            ffi::asmgl_clear_color(color.0 as f32 / 255.0,
                                   color.1 as f32 / 255.0,
                                   color.2 as f32 / 255.0,
                                   color.3 as f32 / 255.0);
        }
    }

    pub fn clear_depth(&mut self, depth: f32) {
        unsafe {
            ffi::asmgl_clear_depth(depth);
        }
    }

    pub fn clear(&mut self) {
        unsafe { ffi::asmgl_clear() };
    }

    pub fn set_depth_test(&mut self, enable: bool) {
        if enable != self.depth_test_enabled {
            unsafe { ffi::asmgl_set_depth_test(enable as u8) };
            self.depth_test_enabled = enable;
        }
    }

    pub fn set_blend_mode(&mut self, mode: gl::BlendMode) {
        if mode != self.blend_mode {
            unsafe { ffi::asmgl_set_blend_mode(mode as u8) };
            self.blend_mode = mode;
        }
    }

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


enum Multi<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> Multi<T> {
    fn len(&self) -> usize {
        match *self {
            Multi::One(ref x) => 1,
            Multi::Many(ref xs) => xs.len(),
        }
    }

    fn first(&self) -> &T {
        match *self {
            Multi::One(ref x) => x,
            Multi::Many(ref xs) => &xs[0],
        }
    }

    fn first_mut(&mut self) -> &mut T {
        match *self {
            Multi::One(ref mut x) => x,
            Multi::Many(ref mut xs) => &mut xs[0],
        }
    }

    fn iter(&self) -> MultiIter<T> {
        match *self {
            Multi::One(ref x) => MultiIter::One(iter::once(x)),
            Multi::Many(ref xs) => MultiIter::Many(xs.iter()),
        }
    }

    fn iter_mut(&mut self) -> MultiIterMut<T> {
        match *self {
            Multi::One(ref mut x) => MultiIterMut::One(iter::once(x)),
            Multi::Many(ref mut xs) => MultiIterMut::Many(xs.iter_mut()),
        }
    }
}

enum MultiIter<'a, T: 'a> {
    One(iter::Once<&'a T>),
    Many(slice::Iter<'a, T>),
}

impl<'a, T> Iterator for MultiIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match *self {
            MultiIter::One(ref mut x) => x.next(),
            MultiIter::Many(ref mut x) => x.next(),
        }
    }
}

enum MultiIterMut<'a, T: 'a> {
    One(iter::Once<&'a mut T>),
    Many(slice::IterMut<'a, T>),
}

impl<'a, T> Iterator for MultiIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        match *self {
            MultiIterMut::One(ref mut x) => x.next(),
            MultiIterMut::Many(ref mut x) => x.next(),
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

    fn create_texture_impl(&mut self, size: (u16, u16), kind: TextureKind) -> Texture {
        let name = self.inner.run(|ctx| ctx.gen_texture(size, kind));
        Texture {
            inner: self.inner.clone(),
            name: name,
            size: size,
            kind: kind,
        }
    }

    fn load_shader_(ctx: &mut Inner,
                    vert_name: &str,
                    frag_name: &str,
                    defs: &str,
                    output_idx: Option<usize>,
                    uniforms: &[gl::UniformSpec],
                    arrays: &[gl::ArraySpec],
                    textures: &[gl::TextureSpec]) -> Shader_ {
        let name = if let Some(idx) = output_idx {
            ctx.load_shader(vert_name,
                            frag_name,
                            &format!("#define OUTPUT_IDX {}\n{}", idx, defs))
        } else {
            ctx.load_shader(vert_name, frag_name, defs)
        };
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
                if loc < 0 {
                    println!("warning: attrib {} is unused", a.name);
                    continue;
                }
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

        Shader_ {
            name: name,

            uniforms: uniform_vec.into_boxed_slice(),
            attribs: attrib_vec.into_boxed_slice(),
            attrib_mask: attrib_mask,
        }
    }

    fn create_framebuffer_<F>(ctx: &mut Inner,
                              mut save_renderbuffer: F,
                              size: (u16, u16),
                              color: &[gl::Attach<GL>],
                              depth: &Option<gl::Attach<GL>>) -> Framebuffer_
            where F: FnMut(Name<Renderbuffer>) {
        let name = ctx.gen_framebuffer();
        ctx.bind_framebuffer(name);

        // Attach color textures
        for (i, att) in color.iter().enumerate() {
            match *att {
                gl::Attach::Texture(ref tex) => {
                    ctx.framebuffer_texture(tex.name, i as i8);
                },
                gl::Attach::Renderbuffer => {
                    let rb_name = ctx.gen_renderbuffer(size, false);
                    ctx.framebuffer_renderbuffer(rb_name, i as i8);
                    save_renderbuffer(rb_name);
                },
            }
        }

        // Attach depth texture/renderbuffer
        match *depth {
            None => {},
            Some(gl::Attach::Texture(tex)) => {
                ctx.framebuffer_texture(tex.name, -1);
            },
            Some(gl::Attach::Renderbuffer) => {
                let rb_name = ctx.gen_renderbuffer(size, true);
                ctx.framebuffer_renderbuffer(rb_name, -1);
                save_renderbuffer(rb_name);
            },
        }

        Framebuffer_ {
            name: name,
        }
    }
}

impl gl::Context for GL {
    fn havoc(&mut self) {
        self.inner.run(|ctx| ctx.havoc());
    }

    fn check_feature(&self, feature: gl::Feature) -> gl::FeatureStatus {
        use client::platform::gl::Feature::*;
        use client::platform::gl::FeatureStatus::*;

        let exts = self.inner.run_imm(|ctx| ctx.extensions);

        match feature {
            DepthTexture =>
                if exts.contains(WEBGL_depth_texture) { Native }
                else { Unavailable },
            MultiPlaneFramebuffer =>
                if exts.contains(WEBGL_draw_buffers) { Native }
                else { Emulated },
        }
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
                   defs: &str,
                   uniforms: &[gl::UniformSpec],
                   arrays: &[gl::ArraySpec],
                   textures: &[gl::TextureSpec],
                   outputs: gl::OutputSpec) -> Shader {
        let inner = self.inner.clone();

        self.inner.run(|ctx| {
            let multi = if ctx.has(WEBGL_draw_buffers) {
                Multi::One(GL::load_shader_(ctx, vert_name, frag_name, defs, None,
                                            uniforms, arrays, textures))
            } else {
                let mut ss = Vec::with_capacity(outputs.color_planes as usize);
                for i in 0 .. outputs.color_planes as usize {
                    ss.push(GL::load_shader_(ctx, vert_name, frag_name, defs, Some(i),
                                             uniforms, arrays, textures));
                }
                Multi::Many(ss)
            };

            Shader {
                inner: inner,
                multi: multi,

                array0_size: arrays[0].size,
                num_arrays: arrays.len() as u8,
                num_textures: textures.len() as u8,
            }
        })
    }

    type Texture = Texture;

    fn create_texture(&mut self, size: (u16, u16)) -> Texture {
        self.create_texture_impl(size, TextureKind::RGBA)
    }

    fn create_depth_texture(&mut self, size: (u16, u16)) -> Texture {
        self.create_texture_impl(size, TextureKind::Depth)
    }

    fn create_luminance_texture(&mut self, size: (u16, u16)) -> Texture {
        self.create_texture_impl(size, TextureKind::Luminance)
    }

    fn load_texture(&mut self, img_name: &str) -> Texture {
        let mut size = (0, 0);
        let name = self.inner.run(|ctx| ctx.load_texture(img_name, &mut size));
        Texture {
            inner: self.inner.clone(),
            name: name,
            size: size,
            kind: TextureKind::RGBA,
        }
    }

    fn texture_import_HACK(&mut self, name: u32, size: (u16, u16)) -> Texture {
        unimplemented!()
    }


    type Framebuffer = Framebuffer;

    fn create_framebuffer(&mut self,
                          size: (u16, u16),
                          color: &[gl::Attach<GL>],
                          depth: Option<gl::Attach<GL>>) -> Framebuffer {
        let inner = self.inner.clone();
        self.inner.run(|ctx| {
            let mut renderbuffers = Vec::new();

            let multi = if ctx.has(WEBGL_draw_buffers) {
                Multi::One(GL::create_framebuffer_(ctx,
                                                   |n| renderbuffers.push(Renderbuffer {
                                                       inner: inner.clone(),
                                                       name: n,
                                                   }),
                                                   size,
                                                   color,
                                                   &depth))
            } else {
                let mut fbs = Vec::with_capacity(color.len());
                for i in 0 .. color.len() {
                    fbs.push(GL::create_framebuffer_(ctx,
                                                     |n| renderbuffers.push(Renderbuffer {
                                                         inner: inner.clone(),
                                                         name: n,
                                                     }),
                                                     size,
                                                     &color[i .. i + 1],
                                                     &depth))
                }
                Multi::Many(fbs)
            };

            Framebuffer {
                inner: inner,
                multi: multi,

                size: size,
                num_colors: color.len() as u8,
                has_depth: depth.is_some(),
                renderbuffers: renderbuffers.into_boxed_slice(),
            }
        })
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
    multi: Multi<Shader_>,

    array0_size: usize,
    num_arrays: u8,
    num_textures: u8,
}

struct Shader_ {
    name: Name<Shader>,

    uniforms: Box<[Uniform]>,
    attribs: Box<[Attrib]>,
    attrib_mask: u32,
}

impl Shader {
    fn draw(&mut self, args: &DrawArgs<GL>) {
        assert!(args.arrays.len() == self.num_arrays as usize);
        assert!(args.textures.len() == self.num_textures as usize);

        let mut inner = self.inner.clone();
        inner.run(|ctx| {
            // WEBGL_draw_buffers woraround:
            // Instead of drawing N color planes of a single framebuffer with a single shader, we
            // have N single-plane framebuffers and N corresponding shaders.
            //
            // The drawing code here is split between Shader and Shader_.  The code in Shader sets
            // up global state, such as the viewport and blend mode.  Shader_'s code handles the
            // shader-specific parts, such as uniforms, which may have different locations in the
            // different shader variants.

            if let Some(viewport) = args.viewport {
                ctx.viewport(viewport);
            } else if let Some(output) = args.output {
                let (w, h) = <Framebuffer as gl::Framebuffer<GL>>::size(output);
                ctx.viewport(Region::sized(V2::new(w as i32, h as i32)));
            } else {
                // Dunno what to do.  Just leave it as it was, I guess?
            }

            let (start, count) =
                if let Some(ref range) = args.range {
                    (range.start, range.end - range.start)
                } else {
                    (0, args.arrays[0].len() / self.array0_size)
                };

            ctx.set_depth_test(args.depth_test);
            ctx.set_blend_mode(args.blend_mode);

            for (i, t) in args.textures.iter().enumerate() {
                ctx.bind_texture(i, t.name);
            }


            if let Some(output) = args.output {
                for (s, fb) in self.multi.iter_mut().zip(output.multi.iter()) {
                    ctx.bind_framebuffer(fb.name);
                    s.draw(ctx, args, start, count);
                }
            } else {
                assert!(self.multi.len() == 1);
                ctx.bind_framebuffer(NO_FRAMEBUFFER);
                self.multi.first_mut().draw(ctx, args, start, count);
            }
        });
    }
}

impl Shader_ {
    fn draw(&mut self,
            ctx: &mut Inner,
            args: &DrawArgs<GL>,
            start: usize,
            count: usize) {
        assert!(args.uniforms.len() == self.uniforms.len());

        // Plane-specific setup

        ctx.bind_shader(self.name);

        for (u, v) in self.uniforms.iter_mut().zip(args.uniforms.iter()) {
            Shader_::set_uniform_value(ctx, u, v);
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


        // Draw!

        ctx.draw_triangles(start, count);
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
        self.multi.first().uniforms.len()
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
        let mut inner = self.inner.clone();
        inner.run(move |ctx| {
            for s in self.multi.iter() {
                ctx.delete_shader(s.name);
            }
        });
    }
}


pub struct Texture {
    inner: InnerPtr,
    name: Name<Texture>,
    size: (u16, u16),
    kind: TextureKind,
}

impl gl::Texture for Texture {
    fn size(&self) -> (u16, u16) {
        self.size
    }

    fn load(&mut self, data: &[u8]) {
        let name = self.name;
        let size = self.size;
        let kind = self.kind;
        self.inner.run(|ctx| {
            ctx.bind_texture(0, name);
            ctx.texture_image(size, kind, data);
        });
    }

    fn load_partial(&mut self, rect: Region<V2>, data: &[u8]) {
        let name = self.name;
        let off = rect.min;
        let size = rect.size();
        let kind = self.kind;
        self.inner.run(|ctx| {
            ctx.bind_texture(0, name);
            ctx.texture_subimage((off.x as u16,
                                  off.y as u16),
                                 (size.x as u16,
                                  size.y as u16),
                                 kind,
                                 data);
        });
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


fn iter_names<'a, T>(first: &'a Name<T>,
                     rest: &'a [Name<T>])
                     -> iter::Chain<iter::Once<&'a Name<T>>, slice::Iter<'a, Name<T>>> {
    iter::once(first).chain(rest.iter())
}


pub struct Framebuffer {
    inner: InnerPtr,
    multi: Multi<Framebuffer_>,

    size: (u16, u16),
    num_colors: u8,
    has_depth: bool,

    // For ownership purposes only.  The RBs are never used once the FB has been constructed, but
    // we need to keep them somewhere and ensure they get destroyed at an appropriate time.
    renderbuffers: Box<[Renderbuffer]>,
}

pub struct Framebuffer_ {
    name: Name<Framebuffer>,
}

impl gl::Framebuffer<GL> for Framebuffer {
    fn size(&self) -> (u16, u16) {
        self.size
    }

    fn num_color_planes(&self) -> usize { self.num_colors as usize }
    fn has_depth_buffer(&self) -> bool { self.has_depth }

    fn clear(&mut self, color: (u8, u8, u8, u8)) {
        let mut inner = self.inner.clone();
        inner.run(|ctx| {
            ctx.clear_color(color);
            ctx.clear_depth(0.0);
            for fb in self.multi.iter() {
                ctx.bind_framebuffer(fb.name);
                ctx.clear();
            }
        });
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let mut inner = self.inner.clone();
        inner.run(move |ctx| {
            for fb in self.multi.iter() {
                ctx.delete_framebuffer(fb.name);
            }
        });
    }
}


pub struct Renderbuffer {
    inner: InnerPtr,
    name: Name<Renderbuffer>,
}

impl Drop for Renderbuffer {
    fn drop(&mut self) {
        let name = self.name;
        self.inner.run(move |ctx| {
            ctx.delete_renderbuffer(name);
        });
    }
}
