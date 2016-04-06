use std::prelude::v1::*;
use std::cell::{Cell, UnsafeCell};
use std::mem;
use std::str::FromStr;

use client::platform::*;
use client::platform::gl::*;


mod ffi {
    extern "C" {
        pub fn asmgl_gen_buffer() -> u32;
        pub fn asmgl_delete_buffer(name: u32);
        pub fn asmgl_bind_buffer_array(name: u32);
        pub fn asmgl_bind_buffer_index(name: u32);
        pub fn asmgl_buffer_data_alloc(len: usize);
        pub fn asmgl_buffer_subdata(offset: usize, ptr: *const u8, len: usize);

        pub fn ap_config_get(key_ptr: *const u8,
                             key_len: usize,
                             value_len_p: *mut usize) -> *mut u8;
        pub fn ap_config_set(key_ptr: *const u8,
                             key_len: usize,
                             value_ptr: *const u8,
                             value_len: usize);
        pub fn ap_config_clear(key_ptr: *const u8,
                               key_len: usize);
    }
}


pub struct AsmPlatform {
    config: AsmConfig,
    gl: GL,
}

impl AsmPlatform {
    pub fn new() -> AsmPlatform {
        AsmPlatform {
            config: AsmConfig,
            gl: GL::new(),
        }
    }
}

impl Platform for AsmPlatform {
    type GL = GL;
    fn gl(&mut self) -> &mut GL { &mut self.gl
    }

    type Config = AsmConfig;
    fn config(&self) -> &AsmConfig { &self.config }
    fn config_mut(&mut self) -> &mut AsmConfig { &mut self.config }
}


// Config

pub struct AsmConfig;

impl Config for AsmConfig {
    fn get_int(&self, key: ConfigKey) -> i64 {
        match i64::from_str(&self.get_str(key)) {
            Ok(i) => i,
            Err(_) => 0,
        }
    }

    fn set_int(&mut self, key: ConfigKey, value: i64) {
        self.set_str(key, &value.to_string());
    }

    fn get_str(&self, key: ConfigKey) -> String {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();

        unsafe {
            let mut value_len = 0;
            let value_ptr = ffi::ap_config_get(key_bytes.as_ptr(),
                                               key_bytes.len(),
                                               &mut value_len);
            String::from_raw_parts(value_ptr, value_len, value_len)
        }
    }

    fn set_str(&mut self, key: ConfigKey, value: &str) {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();
        let value_bytes = value.as_bytes();

        unsafe {
            ffi::ap_config_set(key_bytes.as_ptr(),
                               key_bytes.len(),
                               value_bytes.as_ptr(),
                               value_bytes.len());
        }
    }

    fn clear(&mut self, key: ConfigKey) {
        let key_str = key.to_string();
        let key_bytes = key_str.as_bytes();

        unsafe {
            ffi::ap_config_clear(key_bytes.as_ptr(),
                                 key_bytes.len());
        }
    }
}


// GlContext

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
