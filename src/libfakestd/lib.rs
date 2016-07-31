#![crate_name = "fakestd"]
#![no_std]
#![feature(
    alloc,
    allow_internal_unstable,
    collections,
    collections_bound,
    core_intrinsics,
    macro_reexport,
    raw,
    slice_concat_ext,
    unicode,
)]

// Avoid name collision between libcollections and the std::collections module
extern crate alloc;
extern crate rustc_unicode;
#[macro_reexport(format, vec)]
extern crate collections as collections_;

#[macro_reexport(print, println, log, warn, error)]
extern crate asmrt;
extern crate asmmalloc;     // Pull in the asm.js #[allocator] crate


// Needed for macros
pub use asmrt::{raw_print, raw_println, raw_println_warn, raw_println_err};


// The `pub use`s below match the list in libstd/lib.rs
pub use core::{
    any,
    cell,
    clone,
    cmp,
    convert,
    default,
    hash,
    intrinsics,
    iter,
    marker,
    mem,
    ops,
    ptr,
    raw,
    result,
    option,

    isize,
    i8,
    i16,
    i32,
    i64,
    usize,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
};

pub use alloc::{
    boxed,
    rc,
};

pub use collections_::{
    borrow,
    fmt,
    slice,
    str,
    string,
    vec,
};

pub use rustc_unicode::{
    char,
};

pub mod collections {
    // From libstd/collections/mod.rs
    pub use collections_::{
        Bound,
        binary_heap, BinaryHeap,
        btree_map, BTreeMap,
        btree_set, BTreeSet,
        linked_list, LinkedList,
        vec_deque, VecDeque,
    };
}

pub mod prelude {
    pub mod v1 {
        // From libstd/prelude/v1/mod.rs
        pub use marker::{Copy, Send, Sized, Sync};
        pub use ops::{Drop, Fn, FnMut, FnOnce};

        pub use mem::drop;

        pub use boxed::Box;
        pub use borrow::ToOwned;
        pub use clone::Clone;
        pub use cmp::{PartialEq, PartialOrd, Eq, Ord};
        pub use convert::{AsRef, AsMut, Into, From};
        pub use default::Default;
        pub use iter::{Iterator, Extend, IntoIterator};
        pub use iter::{DoubleEndedIterator, ExactSizeIterator};
        pub use option::Option::{self, Some, None};
        pub use result::Result::{self, Ok, Err};
        pub use slice::SliceConcatExt;
        pub use string::{String, ToString};
        pub use vec::Vec;
    }
}


// libstd defines macros:
//  panic!      (already defined in libcore; libstd overrides it with a different version)
//  print!
//  println!
//  select!     (depends on libstd multithreading features)
//
// print! and println! are reexported from libasmrt.  The others are not needed.
