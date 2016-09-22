#![crate_name = "common_data"]

#![cfg_attr(asmjs, no_std)]
#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(asmjs)] use std::prelude::v1::*;

extern crate common_util;

use std::hash::{Hash, Hasher, SipHasher};
use std::mem;
use std::slice;
use std::str;
use std::usize;

use common_util::Bytes;


#[derive(Clone, Copy, Debug)]
pub struct FileHeader {
    pub minor: u16,
    pub major: u16,
    pub num_sections: u32,
    _reserved0: u32,
    _reserved1: u32,
}
unsafe impl Bytes for FileHeader {}

#[derive(Clone, Copy, Debug)]
pub struct SectionHeader {
    pub name: [u8; 8],
    pub offset: u32,
    pub len: u32,
}
unsafe impl Bytes for SectionHeader {}


pub unsafe trait Section {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const Self;
}

unsafe impl<T: Bytes> Section for [T] {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const [T] {
        slice::from_raw_parts(ptr as *const T,
                              len / mem::size_of::<T>())
    }
}

unsafe impl Section for str {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const str {
        let bytes = <[u8] as Section>::from_bytes(ptr, len);
        str::from_utf8(&*bytes).unwrap()
    }
}


#[macro_export]
macro_rules! gen_data {
    (version = $version:expr;
     $($name:ident ($sect_name:pat): $ty:ty,)*) => {
        pub struct Data {
            // `raw` is never referenced directly, but holds ownership for the other fields.
            #[allow(dead_code)]
            raw: Box<[u8]>,

            $( $name: *const $ty, )*
        }

        impl Data {
            pub fn new(raw: Box<[u8]>) -> Data {
                $( let mut $name: Option<*const $ty> = None; )*

                unsafe {
                    let ptr = raw.as_ptr();
                    assert!(ptr as usize & 7 == 0,
                            "raw data allocation must be 8-byte aligned");

                    let header = &*(ptr as *const $crate::FileHeader);
                    let version = (header.major, header.minor);
                    assert!(version == $version,
                            "unsupported data file version (got {:?}, need {:?}",
                            version, $version);

                    let section_start = ptr.offset(mem::size_of::<$crate::FileHeader>() as isize)
                                        as *const $crate::SectionHeader;
                    let sections = slice::from_raw_parts(section_start,
                                                         header.num_sections as usize);

                    for s in sections {
                        match &s.name {
                            $(
                                $sect_name => {
                                    $name = Some(<$ty as $crate::Section>::from_bytes(
                                        ptr.offset(s.offset as isize),
                                        s.len as usize));
                                },
                            )*

                            _ => {
                                warn!("unknown data section: {:?}", s.name);
                            },
                        }
                    }
                }

                Data {
                    raw: raw,
                    $( $name: $name.expect(
                        concat!("missing section: ", stringify!($sect_name))), )*
                }
            }

            $(
                pub fn $name<'a>(&'a self) -> &'a $ty {
                    unsafe { &*self.$name }
                }
            )*
        }
    };
}


// CHD perfect hash function utilities

/// Parameters for the CHD perfect hash function.
#[derive(Debug)]
pub struct ChdParams<T> {
    /// `m` - the modulus = number of buckets for the main hash table.
    m: u32,
    /// `l_i` - the key used for the `i`^th bucket of the intermediate table.
    l: [T],
}

unsafe impl<T: Bytes> Section for ChdParams<T> {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const ChdParams<T> {
        let dummy_slice: *const [u8] = slice::from_raw_parts(4096 as *const u8, 0);
        let dummy_params: *const ChdParams<T> = mem::transmute(dummy_slice);
        let offset = mem::size_of_val(&*dummy_params);

        let adj_len = len - offset;
        let slice = slice::from_raw_parts(ptr as *const T,
                                          adj_len / mem::size_of::<T>());
        let params: &ChdParams<T> = mem::transmute(slice);
        // Hash table moduli must be powers of two.
        fn is_power_of_two(x: usize) -> bool { x & (x - 1) == 0 }
        assert!(is_power_of_two(params.m as usize));
        assert!(is_power_of_two(params.l.len()));
        params
    }
}

/// Apply the CHD-derived perfect hash function to look up the index in `table` for the given key.
pub fn chd_lookup<H: ?Sized, T, P>(key: &H, table: &[T], params: &ChdParams<P>) -> Option<T>
        where H: Hash,
              T: Copy,
              P: Into<u64>+Copy {
    let mut h1 = SipHasher::new_with_keys(0x123456, 0xfedcba);
    key.hash(&mut h1);
    // params.l.len() is a power of two, so use & instead of %
    let idx1 = h1.finish() as usize & (params.l.len() - 1);
    let l: u64 = params.l[idx1].into();

    let mut h2 = SipHasher::new_with_keys(0x123456 + l, 0xfedcba - l);
    key.hash(&mut h2);
    let idx2 = h2.finish() as usize & (params.m as usize - 1);
    table.get(idx2).map(|&x| x)
}

