use std::collections::HashMap;
use std::hash::Hash;
use std::io::{self, Read, Write};
use std::mem;
use std::u16;

use common_types::*;
use common_util::bytes::{ReadBytes, WriteBytes};
use physics::v3::*;

use types::*;


pub trait ReadFrom: Sized {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self>;
}

pub trait WriteTo {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

pub trait Size {
    fn size(&self) -> usize;
}


macro_rules! bytes_impls {
    ($($T:ty,)*) => {
        $(
            impl ReadFrom for $T {
                fn read_from<R: Read>(r: &mut R) -> io::Result<$T> {
                    r.read_bytes()
                }
            }

            impl WriteTo for $T {
                fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
                    w.write_bytes(*self)
                }
            }

            impl Size for $T {
                fn size(&self) -> usize {
                    mem::size_of::<$T>()
                }
            }
        )*
    };
}

bytes_impls! {
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
    f32, f64,
    (),

    ClientId,
    EntityId,
    InventoryId,
    PlaneId,
    TerrainChunkId,
    StructureId,

    V2, V3, Region<V2>, Region<V3>,
    LocalPos, LocalOffset, LocalTime,
}


macro_rules! tuple_impls {
    ($( ($($name:ident),*), )*) => {
        $(
            impl<$($name: ReadFrom,)*> ReadFrom for ($($name,)*) {
                fn read_from<R: Read>(r: &mut R) -> io::Result<($($name,)*)> {
                    #![allow(non_snake_case)]
                    $( let $name = try!(ReadFrom::read_from(r)); )*
                    Ok(($($name,)*))
                }
            }

            impl<$($name: WriteTo,)*> WriteTo for ($($name,)*) {
                fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
                    #![allow(non_snake_case)]
                    let ($(ref $name,)*) = *self;
                    $( try!(WriteTo::write_to($name, w)); )*
                    Ok(())
                }
            }

            impl<$($name: Size,)*> Size for ($($name,)*) {
                fn size(&self) -> usize {
                    #![allow(non_snake_case)]
                    let ($(ref $name,)*) = *self;
                    let mut sum = 0;
                    $( sum += Size::size($name); )*
                    sum
                }
            }
        )*
    };
}

tuple_impls! {
    (A),
    (A,B),
    (A,B,C),
    (A,B,C,D),
    (A,B,C,D,E),
    (A,B,C,D,E,F),
    (A,B,C,D,E,F,G),
    (A,B,C,D,E,F,G,H),
    (A,B,C,D,E,F,G,H,I),
    (A,B,C,D,E,F,G,H,I,J),
}


impl<T: ReadFrom> ReadFrom for Vec<T> {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Vec<T>> {
        let count = try!(u16::read_from(r)) as usize;
        let mut v = Vec::with_capacity(count);
        for _ in 0 .. count {
            v.push(try!(T::read_from(r)));
        }
        Ok(v)
    }
}

impl<T: WriteTo> WriteTo for [T] {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        assert!(self.len() <= u16::MAX as usize);
        try!((self.len() as u16).write_to(w));
        for x in self {
            try!(x.write_to(w));
        }
        Ok(())
    }
}

impl<T: Size> Size for [T] {
    fn size(&self) -> usize {
        let mut sum = 0_u16.size();
        for x in self {
            sum += x.size();
        }
        sum
    }
}

impl<T: ReadFrom> ReadFrom for Box<[T]> {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Box<[T]>> {
        Vec::<T>::read_from(r).map(|v| v.into_boxed_slice())
    }
}

impl<T: WriteTo> WriteTo for Vec<T> {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (self as &[T]).write_to(w)
    }
}

impl<T: Size> Size for Vec<T> {
    fn size(&self) -> usize {
        (self as &[T]).size()
    }
}


impl ReadFrom for String {
    fn read_from<R: Read>(r: &mut R) -> io::Result<String> {
        let v = try!(Vec::<u8>::read_from(r));
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(e) => fail!("utf8 decoding failed: {}", e),
        }
    }
}

impl WriteTo for str {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.as_bytes().write_to(w)
    }
}

impl Size for str {
    fn size(&self) -> usize {
        0_u16.size() + self.len()
    }
}

impl ReadFrom for Box<str> {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Box<str>> {
        String::read_from(r).map(|v| v.into_boxed_str())
    }
}

impl WriteTo for String {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (self as &str).write_to(w)
    }
}

impl Size for String {
    fn size(&self) -> usize {
        (self as &str).size()
    }
}


impl<'a, T: WriteTo> WriteTo for &'a T {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        WriteTo::write_to(*self, w)
    }
}

impl<'a, T: Size> Size for &'a T {
    fn size(&self) -> usize {
        Size::size(*self)
    }
}


impl<K: ReadFrom+Eq+Hash, V: ReadFrom> ReadFrom for HashMap<K, V> {
    fn read_from<R: Read>(r: &mut R) -> io::Result<HashMap<K, V>> {
        let count = try!(u16::read_from(r)) as usize;
        let mut h = HashMap::with_capacity(count);
        for _ in 0 .. count {
            let k = try!(K::read_from(r));
            let v = try!(V::read_from(r));
            h.insert(k, v);
        }
        Ok(h)
    }
}

impl<K: WriteTo+Eq+Hash, V: WriteTo> WriteTo for HashMap<K, V> {
    fn write_to<W: Write>(&self, w: &mut W) -> ::std::io::Result<()> {
        assert!(self.len() <= u16::MAX as usize);
        try!((self.len() as u16).write_to(w));
        for (k, v) in self {
            try!(k.write_to(w));
            try!(v.write_to(w));
        }
        Ok(())
    }
}

impl<K: Size+Eq+Hash, V: Size> Size for HashMap<K, V> {
    fn size(&self) -> usize {
        let mut sum = 0_u16.size();
        for (k, v) in self {
            sum += k.size();
            sum += v.size();
        }
        sum
    }
}
