use std::io;
use std::mem;
use std::raw;

use libserver_types::*;

use ReadExact;


/// Marker trait for types that can be safely manipulated as a string of bytes.  This means a type
/// `T` can implement `Bytes` only if it has no safety-critical invariants, so that any sequence of
/// bytes of appropriate length can safely be interpreted as a `T`.  For example, the following
/// types may implement bytes:
///
///  - All primitive integer and floating point types implement `Bytes`.  Any sequence of 4 bytes
///    is a valid `u32`, for example.
///  - A tuple or struct containing only `Bytes` elements can implement `Bytes`.
///
/// And these cannot:
///
///  - Safe pointers (`&T`, `Box<T>`) do not implement `Bytes` because they maintain the invariant
///    that the pointer is non-null (among others).
///  - `bool` does not implement `Bytes` because only 0 and 1 are legal values.
///  - No `enum` can implement `Bytes`, because the discriminant is restricted to a particular
///    range of values.
///  - Types with destructors cannot implement `Bytes` since they may have a drop flag, whose
///    invariants are unspecified (but probably it must not be zero).
pub unsafe trait Bytes: Copy { }

macro_rules! bytes_primitive_impls {
    ($($ty:ty),*) => {
        $( unsafe impl Bytes for $ty { } )*
    }
}

bytes_primitive_impls!(
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
    f32, f64,
    ()
    // NB: `bool` is not `Bytes` because it's 8 bits wide but only `0` and `1` are valid.
);

macro_rules! bytes_tuple_impl {
    () => {};
    ($($name:ident)*) => {
        unsafe impl<$($name: Bytes),*> Bytes for ($($name,)*) { }
    };
}
bytes_tuple_impl!(A);
bytes_tuple_impl!(A B);
bytes_tuple_impl!(A B C);
bytes_tuple_impl!(A B C D);
bytes_tuple_impl!(A B C D E);
bytes_tuple_impl!(A B C D E F);
bytes_tuple_impl!(A B C D E F G);
bytes_tuple_impl!(A B C D E F G H);
bytes_tuple_impl!(A B C D E F G H I);
bytes_tuple_impl!(A B C D E F G H I J);
bytes_tuple_impl!(A B C D E F G H I J K);
bytes_tuple_impl!(A B C D E F G H I J K L);

unsafe impl Bytes for V2 { }
unsafe impl Bytes for V3 { }

unsafe impl Bytes for ClientId {}
unsafe impl Bytes for EntityId {}
unsafe impl Bytes for InventoryId {}
unsafe impl Bytes for PlaneId {}
unsafe impl Bytes for TerrainChunkId {}
unsafe impl Bytes for StructureId {}

unsafe impl<Id: Copy> Bytes for Stable<Id> {}


pub trait WriteBytes {
    unsafe fn write_as_bytes<T: Copy>(&mut self, x: T) -> io::Result<()>;
    fn write_bytes<T: Bytes>(&mut self, x: T) -> io::Result<()>;
    fn write_bytes_slice<T: Bytes>(&mut self, x: &[T]) -> io::Result<()>;
}

impl<W: io::Write> WriteBytes for W {
    unsafe fn write_as_bytes<T: Copy>(&mut self, x: T) -> io::Result<()> {
        let slice = mem::transmute(raw::Slice {
            data: &x as *const T as *const u8,
            len: mem::size_of::<T>(),
        });
        self.write_all(slice)
    }

    fn write_bytes<T: Bytes>(&mut self, x: T) -> io::Result<()> {
        unsafe { self.write_as_bytes(x) }
    }

    fn write_bytes_slice<T: Bytes>(&mut self, x: &[T]) -> io::Result<()> {
        let slice = unsafe {
            mem::transmute(raw::Slice {
                data: x.as_ptr() as *const u8,
                len: x.len() * mem::size_of::<T>(),
            })
        };
        self.write_all(slice)
    }
}

pub trait ReadBytes {
    unsafe fn read_as_bytes<T: Copy>(&mut self) -> io::Result<T>;
    fn read_bytes<T: Bytes>(&mut self) -> io::Result<T>;
    fn read_bytes_slice<T: Bytes>(&mut self, x: &mut [T]) -> io::Result<()>;
}

impl<R: io::Read> ReadBytes for R {
    unsafe fn read_as_bytes<T: Copy>(&mut self) -> io::Result<T> {
        let mut x = mem::zeroed();
        let slice = mem::transmute(raw::Slice {
            data: &mut x as *mut T as *mut u8,
            len: mem::size_of::<T>(),
        });
        try!(self.read_exact(slice));
        Ok(x)
    }

    fn read_bytes<T: Bytes>(&mut self) -> io::Result<T> {
        unsafe { self.read_as_bytes() }
    }

    fn read_bytes_slice<T: Bytes>(&mut self, x: &mut [T]) -> io::Result<()> {
        let slice = unsafe {
            mem::transmute(raw::Slice {
                data: x.as_mut_ptr() as *mut u8,
                len: x.len() * mem::size_of::<T>(),
            })
        };
        try!(self.read_exact(slice));
        Ok(())
    }
}
