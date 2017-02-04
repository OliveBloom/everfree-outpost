use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::vec::Vec;


const SMALL_VEC_WORDS: usize = 3;

type Storage = [u64; SMALL_VEC_WORDS];

pub struct SmallVec<T> {
    len: usize,
    data: [u64; SMALL_VEC_WORDS],
    _marker0: PhantomData<T>,
}

struct SmallVecInterp<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
    is_large: bool,
}

fn small_limit<T>() -> usize {
    SMALL_VEC_WORDS * mem::size_of::<u64>() / mem::size_of::<T>()
}

impl<T> SmallVec<T> {
    unsafe fn to_interp(&self) -> SmallVecInterp<T> {
        if self.len <= small_limit::<T>() {
            SmallVecInterp {
                ptr: &self.data as *const _ as *mut T,
                len: self.len,
                cap: small_limit::<T>(),
                is_large: false,
            }
        } else {
            SmallVecInterp {
                ptr: self.data[0] as *mut T,
                len: self.len,
                cap: self.data[1] as usize,
                is_large: true,
            }
        }
    }

    unsafe fn from_interp(&mut self, interp: SmallVecInterp<T>) {
        if !interp.is_large {
            assert!(interp.len <= small_limit::<T>());
            self.len = interp.len;
            // Nothing else to do.  self.data was updated in-place.
        } else {
            assert!(interp.len > small_limit::<T>());
            self.len = interp.len;
            self.data[0] = interp.ptr as u64;
            self.data[1] = interp.cap as u64;
        }
    }

    pub fn new() -> SmallVec<T> {
        unsafe { mem::zeroed() }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        unsafe { self.to_interp().cap }
    }

    pub fn push(&mut self, val: T) {
        if self.len() == small_limit::<T>() {
            self.to_large_push(val);
        } else {
            let mut interp = unsafe { self.to_interp() };
            interp.push(val);
            unsafe { self.from_interp(interp) };
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() == small_limit::<T>() + 1 {
            Some(self.to_small_pop())
        } else {
            let mut interp = unsafe { self.to_interp() };
            let result = interp.pop();
            unsafe { self.from_interp(interp) };
            result
        }
    }

    pub fn clear(&mut self) {
        let mut interp = unsafe { self.to_interp() };
        interp.clear();
        unsafe { self.from_interp(interp) };
    }

    pub fn swap_remove(&mut self, idx: usize) -> T {
        let len = self.len();
        self.as_mut_slice().swap(idx, len - 1);
        self.pop().unwrap()
    }

    pub fn as_ptr(&self) -> *const T {
        unsafe { self.to_interp().ptr as *const T }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.to_interp().ptr }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.as_ptr(), self.len())
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
        }
    }

    fn to_large_push(&mut self, val: T) {
        unsafe {
            // Move all elements of `self` into a vector, then push `val`.
            let mut v = Vec::with_capacity(self.len() * 2 + 1);
            let mut interp = self.to_interp();
            for i in 0..interp.len {
                let val = ptr::read(interp.ptr.offset(i as isize));
                v.push(val);
            }
            v.push(val);

            // Update `self` with the new large interpretation.
            interp.is_large = true;
            interp.from_vec(v);
            self.from_interp(interp);
        }
    }

    fn to_small_pop(&mut self) -> T {
        // Empty `self` into a vector.
        let mut v = unsafe {
            let mut interp = self.to_interp();
            let v = interp.to_vec();
            interp.is_large = false;
            // It's easier to empty `self` and obtain a fresh `interp` than to turn a large interp
            // into a small one.
            self.from_interp(interp);
            v
        };

        // Obtain `result`, then repopulate `self` from the contents of the vector.
        let result = v.pop().unwrap();
        unsafe {
            let mut interp = self.to_interp();
            for val in v.into_iter() {
                interp.push(val);
            }
            self.from_interp(interp);
        }
        result
    }
}

impl<T> Drop for SmallVec<T> {
    fn drop(&mut self) {
        let mut interp = unsafe { self.to_interp() };
        if !interp.is_large {
            for i in 0..interp.len {
                unsafe { ptr::read(interp.ptr.offset(i as isize)) };
            }
        } else {
            unsafe { interp.to_vec() };
        }
        self.len = 0;
    }
}


impl<T> SmallVecInterp<T> {
    #[inline]
    fn push(&mut self, val: T) {
        if !self.is_large {
            assert!(self.len < self.cap);
            unsafe { ptr::write(self.ptr.offset(self.len as isize), val); }
            self.len += 1;
        } else {
            let mut v = unsafe { self.to_vec() };
            v.push(val);
            unsafe { self.from_vec(v) };
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        if !self.is_large {
            if self.len == 0 {
                None
            } else {
                self.len -= 1;
                Some(unsafe { ptr::read(self.ptr.offset(self.len as isize)) })
            }
        } else {
            assert!(self.len >= small_limit::<T>());
            let mut v = unsafe { self.to_vec() };
            let result = v.pop();
            unsafe { self.from_vec(v) };
            result
        }
    }

    #[inline]
    fn clear(&mut self) {
        if !self.is_large {
            while self.len > 0 {
                unsafe {
                    // Drop the item.  Decrement first to avoid double-dropping if a panic occurs
                    // partway through this drop.
                    self.len -= 1;
                    ptr::read(self.ptr.offset(self.len as isize))
                };
            }
        } else {
            let mut v = unsafe { self.to_vec() };
            v.clear();
            unsafe { self.from_vec(v) };
        }
    }

    #[inline]
    unsafe fn to_vec(&mut self) -> Vec<T> {
        let vec = Vec::from_raw_parts(self.ptr, self.len, self.cap);
        self.len = 0;
        self.ptr = ptr::null_mut();
        self.cap = 0;
        vec
    }

    #[inline]
    unsafe fn from_vec(&mut self, v: Vec<T>) {
        self.ptr = v.as_ptr() as *mut T;
        self.len = v.len();
        self.cap = v.capacity();
        mem::forget(v);
    }
}


impl<T> Deref for SmallVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for SmallVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
