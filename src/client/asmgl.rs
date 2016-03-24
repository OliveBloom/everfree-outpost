use std::prelude::v1::*;
use std::cell::{Cell, UnsafeCell};
use std::mem;
use client::gl::*;


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


struct Inner {
    cur_array_buffer: u32,
    cur_index_buffer: u32,
}

impl Inner {
    fn new() -> Inner {
        Inner {
            cur_array_buffer: 0,
            cur_index_buffer: 0,
        }
    }
}


mod ffi {
    extern "C" {
        pub fn asmgl_gen_buffer() -> u32;
        pub fn asmgl_delete_buffer(name: u32);
        pub fn asmgl_bind_buffer_array(name: u32);
        pub fn asmgl_bind_buffer_index(name: u32);
        pub fn asmgl_buffer_data_alloc(len: usize);
        pub fn asmgl_buffer_subdata(offset: usize, ptr: *const u8, len: usize);
    }
}


pub struct GL {
    inner: InnerPtr,
}

impl GL {
    pub fn new() -> GL {
        GL { inner: InnerPtr::new() }
    }
}

impl GlContext for GL {
    type Buffer = Buffer;

    fn create_buffer(&mut self) -> Buffer {
        let name = unsafe { ffi::asmgl_gen_buffer() };
        Buffer {
            inner: self.inner.clone(),
            len: 0,
            name: name,
        }
    }
}


pub struct Buffer {
    inner: InnerPtr,
    len: usize,
    name: u32,
}

impl Buffer {
    pub fn name(&self) -> u32 {
        self.name
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl GlBuffer for Buffer {
    fn bind(&mut self, target: BufferTarget) {
        let name = self.name;
        self.inner.run(|ctx| {
            match target {
                BufferTarget::Array => 
                    if ctx.cur_array_buffer != name {
                        unsafe { ffi::asmgl_bind_buffer_array(name) };
                        // TODO: uncomment this once asmgl is the sole owner of the ARRAY_BUFFER
                        // attachment point
                        //ctx.cur_array_buffer = name;
                    },
                BufferTarget::Index => 
                    if ctx.cur_index_buffer != name {
                        unsafe { ffi::asmgl_bind_buffer_index(name) };
                        //ctx.cur_index_buffer = name;
                    },
            }
        });
    }

    fn alloc(&mut self, len: usize) {
        self.bind(BufferTarget::Array);
        unsafe { ffi::asmgl_buffer_data_alloc(len) };
        self.len = len;
    }

    fn load(&mut self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() <= self.len);
        self.bind(BufferTarget::Array);
        unsafe { ffi::asmgl_buffer_subdata(offset, data.as_ptr(), data.len()) };
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let name = self.name;
        self.inner.run(|ctx| {
            unsafe { ffi::asmgl_delete_buffer(name) };
            if ctx.cur_array_buffer == name {
                ctx.cur_array_buffer = 0;
            }
            if ctx.cur_index_buffer == name {
                ctx.cur_index_buffer = 0;
            }
        });
    }
}
