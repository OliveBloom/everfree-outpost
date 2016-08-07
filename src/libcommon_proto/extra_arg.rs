use std::prelude::v1::*;
#[cfg(asmjs)] use std::collections::BTreeMap;
#[cfg(not(asmjs))] use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::mem;

use wire::{ReadFrom, WriteTo, Size};


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(not(asmjs), derive(Hash))]
pub enum SimpleArg {
    Int(i32),
    Str(String),
}

impl SimpleArg {
    pub fn into_extra_arg(self) -> ExtraArg {
        match self {
            SimpleArg::Int(x) => ExtraArg::Int(x),
            SimpleArg::Str(x) => ExtraArg::Str(x),
        }
    }
}

#[cfg(asmjs)] pub type Map<K, V> = BTreeMap<K, V>;
#[cfg(not(asmjs))] pub type Map<K, V> = HashMap<K, V>;

#[derive(Clone, Debug)]
pub enum ExtraArg {
    Int(i32),
    Str(String),
    List(Vec<ExtraArg>),
    Map(Map<SimpleArg, ExtraArg>),
}

impl ExtraArg {
    pub fn into_simple_arg(self) -> Result<SimpleArg, ExtraArg> {
        match self {
            ExtraArg::Int(x) => Ok(SimpleArg::Int(x)),
            ExtraArg::Str(x) => Ok(SimpleArg::Str(x)),
            e => Err(e),
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum ArgTag {
    Int = 0,
    Str = 1,
    List = 2,
    Map = 3,
}

impl ReadFrom for ArgTag {
    fn read_from<R: Read>(r: &mut R) -> io::Result<ArgTag> {
        let raw = try!(u8::read_from(r));
        match raw {
            0 => Ok(ArgTag::Int),
            1 => Ok(ArgTag::Str),
            2 => Ok(ArgTag::List),
            3 => Ok(ArgTag::Map),
            _ => fail!("bad ArgTag: {}", raw),
        }
    }
}

impl WriteTo for ArgTag {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (*self as u8).write_to(w)
    }
}

impl Size for ArgTag {
    fn size(&self) -> usize {
        mem::size_of::<u8>()
    }
}


impl ReadFrom for ExtraArg {
    fn read_from<R: Read>(r: &mut R) -> io::Result<ExtraArg> {
        match try!(ArgTag::read_from(r)) {
            ArgTag::Int => {
                let i = try!(i32::read_from(r));
                Ok(ExtraArg::Int(i))
            },
            ArgTag::Str => {
                let s = try!(String::read_from(r));
                Ok(ExtraArg::Str(s))
            },
            ArgTag::List => {
                let v = try!(Vec::<ExtraArg>::read_from(r));
                Ok(ExtraArg::List(v))
            },
            ArgTag::Map => {
                let h = try!(Map::<SimpleArg, ExtraArg>::read_from(r));
                Ok(ExtraArg::Map(h))
            },
        }
    }
}

impl ReadFrom for SimpleArg {
    fn read_from<R: Read>(r: &mut R) -> io::Result<SimpleArg> {
        let x = try!(ExtraArg::read_from(r));
        match x.into_simple_arg() {
            Ok(x) => Ok(x),
            Err(_) => fail!("unexpected non-simple ExtraArg"),
        }
    }
}

impl WriteTo for ExtraArg {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match *self {
            ExtraArg::Int(i) =>
                (ArgTag::Int, i).write_to(w),
            ExtraArg::Str(ref s) =>
                (ArgTag::Str, s).write_to(w),
            ExtraArg::List(ref v) =>
                (ArgTag::List, v).write_to(w),
            ExtraArg::Map(ref h) =>
                (ArgTag::Map, h).write_to(w),
        }
    }
}

impl WriteTo for SimpleArg {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match *self {
            SimpleArg::Int(i) =>
                (ArgTag::Int, i).write_to(w),
            SimpleArg::Str(ref s) =>
                (ArgTag::Str, s).write_to(w),
        }
    }
}

impl Size for ExtraArg {
    fn size(&self) -> usize {
        match *self {
            ExtraArg::Int(i) => 1 + i.size(),
            ExtraArg::Str(ref s) => 1 + s.size(),
            ExtraArg::List(ref v) => 1 + v.size(),
            ExtraArg::Map(ref h) => 1 + h.size(),
        }
    }
}

impl Size for SimpleArg {
    fn size(&self) -> usize {
        match *self {
            SimpleArg::Int(i) => 1 + i.size(),
            SimpleArg::Str(ref s) => 1 + s.size(),
        }
    }
}
