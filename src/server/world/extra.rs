use std::collections::HashMap;
use std::collections::hash_map;
use std::io;
use std::ptr;
use std::slice;

use types::*;
use util::Convert;

use world::fragment::Fragment;
use world::save::{self, Reader};
use world::bundle::export::{Export, Exporter};
use world::bundle::import::{Import, Importer};
use world::bundle::write::{self, WriteExt};


macro_rules! with_value_variants {
    ($mac:ident!($($args:tt)*)) => {
        $mac! {
            $($args)*,
            {
                Bool(bool),
                Int(i64),
                Float(f64),
                Str(String),

                ClientId(ClientId),
                EntityId(EntityId),
                InventoryId(InventoryId),
                PlaneId(PlaneId),
                TerrainChunkId(TerrainChunkId),
                StructureId(StructureId),

                StableClientId(Stable<ClientId>),
                StableEntityId(Stable<EntityId>),
                StableInventoryId(Stable<InventoryId>),
                StablePlaneId(Stable<PlaneId>),
                StableTerrainChunkId(Stable<TerrainChunkId>),
                StableStructureId(Stable<StructureId>),

                V2(V2),
                V3(V3),
                Region2(Region<V2>),
                Region3(Region<V3>),
            }
        }
    };
}

macro_rules! mk_repr {
    ($Repr:ident, { $($vname:ident($vty:ty),)* }) => {
        #[derive(Clone, Debug)]
        enum $Repr {
            Null,
            Array(Vec<Repr>),
            Hash(HashMap<String, Repr>),
            $($vname($vty),)*
        }
    };
}
with_value_variants!(mk_repr!(Repr));

macro_rules! mk_value {
    ($Value:ident, $Repr:ident, { $($vname:ident($vty:ty),)* }) => {
        #[derive(Clone, Debug)]
        pub enum $Value {
            Null,
            $($vname($vty),)*
        }

        impl $Value {
            fn from_repr(repr: $Repr) -> Option<$Value> {
                match repr {
                    $Repr::Null => Some($Value::Null),
                    $Repr::Array(_) => None,
                    $Repr::Hash(_) => None,
                    $(
                        $Repr::$vname(val) => Some($Value::$vname(val)),
                    )*
                }
            }

            fn to_repr(self) -> $Repr {
                match self {
                    $Value::Null => $Repr::Null,
                    $(
                        $Value::$vname(val) => $Repr::$vname(val),
                    )*
                }
            }
        }
    };
}
with_value_variants!(mk_value!(Value, Repr));


pub enum View<'a> {
    Value(Value),
    Array(ArrayView<'a>),
    Hash(HashView<'a>),
}

impl<'a> View<'a> {
    fn from_repr(repr: &'a Repr) -> View<'a> {
        match *repr {
            Repr::Array(ref v) => View::Array(ArrayView::new(v)),
            Repr::Hash(ref h) => View::Hash(HashView::new(h)),
            ref val => View::Value(Value::from_repr(val.clone()).unwrap()),
        }
    }
}

pub enum ViewMut<'a> {
    Value(Value),
    Array(ArrayViewMut<'a>),
    Hash(HashViewMut<'a>),
}

impl<'a> ViewMut<'a> {
    fn from_repr(repr: &'a mut Repr) -> ViewMut<'a> {
        match *repr {
            Repr::Array(ref mut v) => ViewMut::Array(ArrayViewMut::new(v)),
            Repr::Hash(ref mut h) => ViewMut::Hash(HashViewMut::new(h)),
            ref val => ViewMut::Value(Value::from_repr(val.clone()).unwrap()),
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct ArrayView<'a> {
    v: &'a Vec<Repr>,
}

impl<'a> ArrayView<'a> {
    fn new(v: &'a Vec<Repr>) -> ArrayView<'a> {
        ArrayView { v: v }
    }

    pub fn get(self, idx: usize) -> View<'a> {
        View::from_repr(&self.v[idx])
    }

    pub fn len(self) -> usize {
        self.v.len()
    }

    pub fn iter(self) -> ArrayViewIter<'a> {
        ArrayViewIter { inner: self.v.iter() }
    }
}

#[derive(Debug)]
pub struct ArrayViewMut<'a> {
    v: &'a mut Vec<Repr>,
}

impl<'a> ArrayViewMut<'a> {
    fn new(v: &'a mut Vec<Repr>) -> ArrayViewMut<'a> {
        ArrayViewMut { v: v }
    }

    pub fn borrow<'b>(&'b mut self) -> ArrayViewMut<'b> {
        ArrayViewMut { v: self.v }
    }

    pub fn get(self, idx: usize) -> View<'a> {
        View::from_repr(&self.v[idx])
    }

    pub fn get_mut(self, idx: usize) -> ViewMut<'a> {
        ViewMut::from_repr(&mut self.v[idx])
    }

    pub fn len(self) -> usize {
        self.v.len()
    }

    pub fn set(self, idx: usize, value: Value) {
        self.v[idx] = value.to_repr();
    }

    pub fn set_array(self, idx: usize) -> ArrayViewMut<'a> {
        let top = self.v;
        top[idx] = Repr::Array(Vec::new());
        if let Repr::Array(ref mut v) = top[idx] {
            ArrayViewMut::new(v)
        } else {
            unreachable!("result of inserting Array should be Array");
        }
    }

    pub fn set_hash(self, idx: usize) -> HashViewMut<'a> {
        let top = self.v;
        top[idx] = Repr::Hash(HashMap::new());
        if let Repr::Hash(ref mut h) = top[idx] {
            HashViewMut::new(h)
        } else {
            unreachable!("result of inserting Hash should be Hash");
        }
    }

    pub fn push(self) {
        self.v.push(Repr::Null);
    }

    pub fn pop(self) {
        self.v.pop();
    }

    pub fn iter(self) -> ArrayViewIter<'a> {
        ArrayViewIter { inner: self.v.iter() }
    }

    pub fn iter_mut(self) -> ArrayViewIterMut<'a> {
        ArrayViewIterMut { inner: self.v.iter_mut() }
    }
}

pub struct ArrayViewIter<'a> {
    inner: slice::Iter<'a, Repr>,
}

impl<'a> Iterator for ArrayViewIter<'a> {
    type Item = View<'a>;

    fn next(&mut self) -> Option<View<'a>> {
        match self.inner.next() {
            Some(x) => Some(View::from_repr(x)),
            None => None,
        }
    }
}

pub struct ArrayViewIterMut<'a> {
    inner: slice::IterMut<'a, Repr>,
}

impl<'a> Iterator for ArrayViewIterMut<'a> {
    type Item = ViewMut<'a>;

    fn next(&mut self) -> Option<ViewMut<'a>> {
        match self.inner.next() {
            Some(x) => Some(ViewMut::from_repr(x)),
            None => None,
        }
    }
}


pub struct HashView<'a> {
    h: &'a HashMap<String, Repr>,
}

impl<'a> HashView<'a> {
    fn new(h: &'a HashMap<String, Repr>) -> HashView<'a> {
        HashView { h: h }
    }

    pub fn get(self, key: &str) -> Option<View<'a>> {
        self.h.get(key).map(View::from_repr)
    }

    pub fn contains(self, key: &str) -> bool {
        self.h.contains_key(key)
    }

    pub fn len(self) -> usize {
        self.h.len()
    }

    pub fn iter(self) -> HashViewIter<'a> {
        HashViewIter { inner: self.h.iter() }
    }
}

pub struct HashViewMut<'a> {
    h: &'a mut HashMap<String, Repr>,
}

impl<'a> HashViewMut<'a> {
    fn new(h: &'a mut HashMap<String, Repr>) -> HashViewMut<'a> {
        HashViewMut { h: h }
    }

    pub fn borrow<'b>(&'b mut self) -> HashViewMut<'b> {
        HashViewMut { h: self.h }
    }

    pub fn get(self, key: &str) -> Option<View<'a>> {
        self.h.get(key).map(View::from_repr)
    }

    pub fn get_mut(self, key: &str) -> Option<ViewMut<'a>> {
        self.h.get_mut(key).map(ViewMut::from_repr)
    }

    pub fn contains(self, key: &str) -> bool {
        self.h.contains_key(key)
    }

    pub fn len(self) -> usize {
        self.h.len()
    }

    pub fn set(self, key: &str, value: Value) {
        self.h.insert(key.to_owned(), value.to_repr());
    }

    pub fn set_array(self, key: &str) -> ArrayViewMut<'a> {
        self.h.insert(key.to_owned(), Repr::Array(Vec::new()));
        if let Repr::Array(ref mut v) = *self.h.get_mut(key).unwrap() {
            ArrayViewMut::new(v)
        } else {
            unreachable!("result of inserting Array should be Array");
        }
    }

    pub fn set_hash(self, key: &str) -> HashViewMut<'a> {
        self.h.insert(key.to_owned(), Repr::Hash(HashMap::new()));
        if let Repr::Hash(ref mut h) = *self.h.get_mut(key).unwrap() {
            HashViewMut::new(h)
        } else {
            unreachable!("result of inserting Hash should be Hash");
        }
    }

    pub fn remove(self, key: &str) {
        self.h.remove(key);
    }

    pub fn iter(self) -> HashViewIter<'a> {
        HashViewIter { inner: self.h.iter() }
    }

    pub fn iter_mut(self) -> HashViewIterMut<'a> {
        HashViewIterMut { inner: self.h.iter_mut() }
    }
}

pub struct HashViewIter<'a> {
    inner: hash_map::Iter<'a, String, Repr>,
}

impl<'a> Iterator for HashViewIter<'a> {
    type Item = (&'a str, View<'a>);

    fn next(&mut self) -> Option<(&'a str, View<'a>)> {
        match self.inner.next() {
            Some((k, v)) => Some((k, View::from_repr(v))),
            None => None,
        }
    }
}

pub struct HashViewIterMut<'a> {
    inner: hash_map::IterMut<'a, String, Repr>,
}

impl<'a> Iterator for HashViewIterMut<'a> {
    type Item = (&'a str, ViewMut<'a>);

    fn next(&mut self) -> Option<(&'a str, ViewMut<'a>)> {
        match self.inner.next() {
            Some((k, v)) => Some((k, ViewMut::from_repr(v))),
            None => None,
        }
    }
}


#[derive(Clone)]
pub struct Extra {
    h: Option<Box<HashMap<String, Repr>>>,
}

impl Extra {
    pub fn new() -> Extra {
        Extra {
            h: None,
        }
    }

    pub fn get(&self, key: &str) -> Option<View> {
        if let Some(ref h) = self.h {
            HashView::new(h).get(key)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<ViewMut> {
        if let Some(ref mut h) = self.h {
            HashViewMut::new(h).get_mut(key)
        } else {
            None
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        if let Some(ref h) = self.h {
            HashView::new(h).contains(key)
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        if let Some(ref h) = self.h {
            h.len()
        } else {
            0
        }
    }

    fn view_mut(&mut self) -> HashViewMut {
        if self.h.is_none() {
            self.h = Some(Box::new(HashMap::new()));
        }
        if let Some(ref mut h) = self.h {
            HashViewMut::new(h)
        } else {
            unreachable!("value after setting to Some should be Some");
        }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.view_mut().set(key, value)
    }

    pub fn set_array(&mut self, key: &str) -> ArrayViewMut {
        self.view_mut().set_array(key)
    }

    pub fn set_hash(&mut self, key: &str) -> HashViewMut {
        self.view_mut().set_hash(key)
    }

    pub fn remove(&mut self, key: &str) {
        let clear = 
            if let Some(ref mut h) = self.h {
                HashViewMut::new(h).remove(key);
                h.len() == 0
            } else {
                // It's already empty.
                return;
            };
        if clear {
            self.h = None;
        }
    }

    pub fn iter(&self) -> ExtraIter {
        if let Some(ref h) = self.h {
            ExtraIter { inner: Some(HashView::new(h).iter()) }
        } else {
            ExtraIter { inner: None }
        }
    }

    pub fn iter_mut(&mut self) -> ExtraIterMut {
        if let Some(ref mut h) = self.h {
            ExtraIterMut { inner: Some(HashViewMut::new(h).iter_mut()) }
        } else {
            ExtraIterMut { inner: None }
        }
    }
}

struct ExtraIter<'a> {
    inner: Option<HashViewIter<'a>>,
}

impl<'a> Iterator for ExtraIter<'a> {
    type Item = (&'a str, View<'a>);

    fn next(&mut self) -> Option<(&'a str, View<'a>)> {
        if let Some(ref mut inner) = self.inner {
            inner.next()
        } else {
            None
        }
    }
}

struct ExtraIterMut<'a> {
    inner: Option<HashViewIterMut<'a>>,
}

impl<'a> Iterator for ExtraIterMut<'a> {
    type Item = (&'a str, ViewMut<'a>);

    fn next(&mut self) -> Option<(&'a str, ViewMut<'a>)> {
        if let Some(ref mut inner) = self.inner {
            inner.next()
        } else {
            None
        }
    }
}


// TODO: copied from script::save
macro_rules! primitive_enum {
    (enum $name:ident: $prim:ty { $($variant:ident = $disr:expr,)* }) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum $name {
            $($variant = $disr,)*
        }

        impl $name {
            pub fn from_primitive(x: $prim) -> Option<$name> {
                match x {
                    $( $disr => Some($name::$variant), )*
                    _ => None,
                }
            }
        }
    };
}

primitive_enum! {
    enum Tag: u8 {
        Null =          0x00,
        Bool =          0x01,
        SmallInt =      0x02,
        LargeInt =      0x03,
        Float =         0x04,
        SmallStr =      0x05,
        LargeStr =      0x06,

        SmallArray =    0x10,
        LargeArray =    0x11,
        SmallHash =     0x12,
        LargeHash =     0x13,

        ClientId =          0x20,
        EntityId =          0x21,
        InventoryId =       0x22,
        PlaneId =           0x23,
        TerrainChunkId =    0x24,
        StructureId =       0x25,

        StableClientId =        0x30,
        StableEntityId =        0x31,
        StableInventoryId =     0x32,
        StablePlaneId =         0x33,
        StableTerrainChunkId =  0x34,
        StableStructureId =     0x35,

        V2 =            0x40,
        V3 =            0x41,
        Region2 =       0x42,
        Region3 =       0x43,
    }
}


pub fn write<W: io::Write>(w: &mut W, e: &Extra) -> write::Result<()> {
    match e.h {
        Some(ref h) => write_hash(w, h),
        None => w.write_val(Tag::Null as u8),
    }
}

fn write_repr<W: io::Write>(w: &mut W, r: &Repr) -> write::Result<()> {
    match *r {
        Repr::Null =>
            w.write_val(Tag::Null as u8),
        Repr::Bool(b) =>
            w.write_val((Tag::Bool as u8, b as u8)),
        Repr::Int(i) =>
            match i.to_i16() {
                Some(small_i) =>
                    w.write_val((Tag::SmallInt as u8, 0u8, small_i)),
                None => {
                    try!(w.write_val(Tag::LargeInt as u8));
                    w.write_val(i)
                },
            },
        Repr::Float(f) => {
            try!(w.write_val(Tag::Float as u8));
            w.write_val(f)
        },
        Repr::Str(ref s) => {
            try!(write_tag_and_len(w, s.len(), Tag::SmallStr, Tag::LargeStr));
            w.write_str_bytes(s)
        },

        Repr::Array(ref v) => write_array(w, v),
        Repr::Hash(ref h) => write_hash(w, h),

        Repr::ClientId(id) => {
            try!(w.write_val(Tag::ClientId as u8));
            w.write_val(id.unwrap())
        },
        Repr::EntityId(id) => {
            try!(w.write_val(Tag::EntityId as u8));
            w.write_val(id.unwrap())
        },
        Repr::InventoryId(id) => {
            try!(w.write_val(Tag::InventoryId as u8));
            w.write_val(id.unwrap())
        },
        Repr::PlaneId(id) => {
            try!(w.write_val(Tag::PlaneId as u8));
            w.write_val(id.unwrap())
        },
        Repr::TerrainChunkId(id) => {
            try!(w.write_val(Tag::TerrainChunkId as u8));
            w.write_val(id.unwrap())
        },
        Repr::StructureId(id) => {
            try!(w.write_val(Tag::StructureId as u8));
            w.write_val(id.unwrap())
        },

        Repr::StableClientId(id) => {
            try!(w.write_val(Tag::StableClientId as u8));
            w.write_val(id.unwrap())
        },
        Repr::StableEntityId(id) => {
            try!(w.write_val(Tag::StableEntityId as u8));
            w.write_val(id.unwrap())
        },
        Repr::StableInventoryId(id) => {
            try!(w.write_val(Tag::StableInventoryId as u8));
            w.write_val(id.unwrap())
        },
        Repr::StablePlaneId(id) => {
            try!(w.write_val(Tag::StablePlaneId as u8));
            w.write_val(id.unwrap())
        },
        Repr::StableTerrainChunkId(id) => {
            try!(w.write_val(Tag::StableTerrainChunkId as u8));
            w.write_val(id.unwrap())
        },
        Repr::StableStructureId(id) => {
            try!(w.write_val(Tag::StableStructureId as u8));
            w.write_val(id.unwrap())
        },

        Repr::V2(v2) => {
            try!(w.write_val(Tag::V2 as u8));
            w.write_val(v2)
        },
        Repr::V3(v3) => {
            try!(w.write_val(Tag::V3 as u8));
            w.write_val(v3)
        },
        Repr::Region2(region2) => {
            try!(w.write_val(Tag::Region2 as u8));
            try!(w.write_val(region2.min));
            try!(w.write_val(region2.max));
            Ok(())
        },
        Repr::Region3(region3) => {
            try!(w.write_val(Tag::Region3 as u8));
            try!(w.write_val(region3.min));
            try!(w.write_val(region3.max));
            Ok(())
        },
    }
}

fn write_array<W: io::Write>(w: &mut W, v: &Vec<Repr>) -> write::Result<()> {
    try!(write_tag_and_len(w, v.len(), Tag::SmallArray, Tag::LargeArray));
    for x in v {
        try!(write_repr(w, x));
    }
    Ok(())
}

fn write_hash<W: io::Write>(w: &mut W, h: &HashMap<String, Repr>) -> write::Result<()> {
    try!(write_tag_and_len(w, h.len(), Tag::SmallHash, Tag::LargeHash));
    for (k, v) in h {
        try!(w.write_str(k));
        try!(write_repr(w, v));
    }
    Ok(())
}

fn write_tag_and_len<W: io::Write>(w: &mut W,
                                   len: usize,
                                   small_tag: Tag,
                                   large_tag: Tag) -> write::Result<()> {
    match len.to_u16() {
        Some(small_len) => {
            try!(w.write_val((small_tag as u8, 0u8, small_len)));
        },
        None => {
            try!(w.write_val(large_tag as u8));
            try!(w.write_count(len));
        },
    }
    Ok(())
}


pub fn read<'d, R: ?Sized, F>(r: &mut R, f: &mut F) -> save::Result<Extra>
        where R: Reader,
              F: Fragment<'d> {
    let r = try!(read_repr(r, f));
    match r {
        Repr::Null => Ok(Extra { h: None }),
        Repr::Hash(h) => Ok(Extra { h: Some(Box::new(h)) }),
        _ => fail!("top-level Repr should be Null or Hash"),
    }
}

fn read_repr<'d, R: ?Sized, F>(r: &mut R, f: &mut F) -> save::Result<Repr>
        where R: Reader,
              F: Fragment<'d> {
    let (raw_tag, a, b): (u8, u8, u16) = try!(r.read());
    let tag = unwrap!(Tag::from_primitive(raw_tag));
    match tag {
        Tag::Null =>
            Ok(Repr::Null),
        Tag::Bool =>
            Ok(Repr::Bool(a != 0)),
        Tag::SmallInt =>
            Ok(Repr::Int(b as i16 as i64)),
        Tag::LargeInt => {
            let i = try!(r.read());
            Ok(Repr::Int(i))
        },
        Tag::Float => {
            let i = try!(r.read());
            Ok(Repr::Int(i))
        },
        Tag::SmallStr =>
            read_str(r, Some(b as usize)).map(Repr::Str),
        Tag::LargeStr =>
            read_str(r, None).map(Repr::Str),

        Tag::SmallArray =>
            read_array(r, f, Some(b as usize)).map(Repr::Array),
        Tag::LargeArray =>
            read_array(r, f, None).map(Repr::Array),
        Tag::SmallHash =>
            read_hash(r, f, Some(b as usize)).map(Repr::Hash),
        Tag::LargeHash =>
            read_hash(r, f, None).map(Repr::Hash),

        Tag::ClientId => {
            let id = try!(r.read_id(f));
            Ok(Repr::ClientId(id))
        },
        Tag::EntityId => {
            let id = try!(r.read_id(f));
            Ok(Repr::EntityId(id))
        },
        Tag::InventoryId => {
            let id = try!(r.read_id(f));
            Ok(Repr::InventoryId(id))
        },
        Tag::PlaneId => {
            let id = try!(r.read_id(f));
            Ok(Repr::PlaneId(id))
        },
        Tag::TerrainChunkId => {
            let id = try!(r.read_id(f));
            Ok(Repr::TerrainChunkId(id))
        },
        Tag::StructureId => {
            let id = try!(r.read_id(f));
            Ok(Repr::StructureId(id))
        },

        Tag::StableClientId => {
            let id = try!(r.read());
            Ok(Repr::StableClientId(Stable::new(id)))
        },
        Tag::StableEntityId => {
            let id = try!(r.read());
            Ok(Repr::StableEntityId(Stable::new(id)))
        },
        Tag::StableInventoryId => {
            let id = try!(r.read());
            Ok(Repr::StableInventoryId(Stable::new(id)))
        },
        Tag::StablePlaneId => {
            let id = try!(r.read());
            Ok(Repr::StablePlaneId(Stable::new(id)))
        },
        Tag::StableTerrainChunkId => {
            let id = try!(r.read());
            Ok(Repr::StableTerrainChunkId(Stable::new(id)))
        },
        Tag::StableStructureId => {
            let id = try!(r.read());
            Ok(Repr::StableStructureId(Stable::new(id)))
        },

        Tag::V2 => {
            let v2 = try!(r.read());
            Ok(Repr::V2(v2))
        },
        Tag::V3 => {
            let v3 = try!(r.read());
            Ok(Repr::V3(v3))
        },
        Tag::Region2 => {
            let min = try!(r.read());
            let max = try!(r.read());
            Ok(Repr::Region2(Region::new(min, max)))
        },
        Tag::Region3 => {
            let min = try!(r.read());
            let max = try!(r.read());
            Ok(Repr::Region3(Region::new(min, max)))
        },
    }
}

fn read_str<R: ?Sized>(r: &mut R,
                       opt_len: Option<usize>) -> save::Result<String>
        where R: Reader {
    let len = match opt_len {
        Some(x) => x,
        None => try!(r.read_count()),
    };
    let s = try!(r.read_str_bytes(len));
    Ok(s)
}

fn read_array<'d, R: ?Sized, F>(r: &mut R,
                                f: &mut F,
                                opt_len: Option<usize>) -> save::Result<Vec<Repr>>
        where R: Reader,
              F: Fragment<'d> {
    let len = match opt_len {
        Some(x) => x,
        None => try!(r.read_count()),
    };
    let mut v = Vec::with_capacity(len);
    for _ in 0 .. len {
        let x = try!(read_repr(r, f));
        v.push(x);
    }
    Ok(v)
}

fn read_hash<'d, R: ?Sized, F>(r: &mut R,
                               f: &mut F,
                               opt_len: Option<usize>) -> save::Result<HashMap<String, Repr>>
        where R: Reader,
              F: Fragment<'d> {
    let len = match opt_len {
        Some(x) => x,
        None => try!(r.read_count()),
    };
    let mut h = HashMap::with_capacity(len);
    for _ in 0 .. len {
        let k = try!(r.read_str());
        let v = try!(read_repr(r, f));
        h.insert(k, v);
    }
    Ok(h)
}


impl Export for Repr {
    fn export_to(&self, e: &mut Exporter) -> Repr {
        use self::Repr::*;
        match *self {
            Null => Null,
            Array(ref v) => Array(v.iter().map(|x| e.export(x)).collect()),
            Hash(ref h) => Hash(h.iter().map(|(k,v)| (k.clone(), e.export(v))).collect()),
            Bool(b) => Bool(b),
            Int(i) => Int(i),
            Float(f) => Float(f),
            Str(ref s) => Str(s.clone()),

            ClientId(id) => ClientId(e.export(&id)),
            EntityId(id) => EntityId(e.export(&id)),
            InventoryId(id) => InventoryId(e.export(&id)),
            PlaneId(id) => PlaneId(e.export(&id)),
            TerrainChunkId(id) => TerrainChunkId(e.export(&id)),
            StructureId(id) => StructureId(e.export(&id)),

            StableClientId(id) => StableClientId(id),
            StableEntityId(id) => StableEntityId(id),
            StableInventoryId(id) => StableInventoryId(id),
            StablePlaneId(id) => StablePlaneId(id),
            StableTerrainChunkId(id) => StableTerrainChunkId(id),
            StableStructureId(id) => StableStructureId(id),

            V2(v) => V2(v),
            V3(v) => V3(v),
            Region2(r) => Region2(r),
            Region3(r) => Region3(r),
        }
    }
}

impl Export for Extra {
    fn export_to(&self, e: &mut Exporter) -> Extra {
        if let Some(ref h) = self.h {
            let mut h2 = HashMap::with_capacity(h.len());
            for (k, v) in h.iter() {
                h2.insert(k.clone(), e.export(v));
            }
            Extra { h: Some(Box::new(h2)) }
        } else {
            Extra { h: None }
        }
    }
}


impl Import for Repr {
    fn import_from(&self, i: &Importer) -> Repr {
        use self::Repr::*;
        match *self {
            Null => Null,
            Array(ref v) => Array(v.iter().map(|x| i.import(x)).collect()),
            Hash(ref h) => Hash(h.iter().map(|(k,v)| (k.clone(), i.import(v))).collect()),
            Bool(b) => Bool(b),
            Int(i) => Int(i),
            Float(f) => Float(f),
            Str(ref s) => Str(s.clone()),

            ClientId(id) => ClientId(i.import(&id)),
            EntityId(id) => EntityId(i.import(&id)),
            InventoryId(id) => InventoryId(i.import(&id)),
            PlaneId(id) => PlaneId(i.import(&id)),
            TerrainChunkId(id) => TerrainChunkId(i.import(&id)),
            StructureId(id) => StructureId(i.import(&id)),

            StableClientId(id) => StableClientId(id),
            StableEntityId(id) => StableEntityId(id),
            StableInventoryId(id) => StableInventoryId(id),
            StablePlaneId(id) => StablePlaneId(id),
            StableTerrainChunkId(id) => StableTerrainChunkId(id),
            StableStructureId(id) => StableStructureId(id),

            V2(v) => V2(v),
            V3(v) => V3(v),
            Region2(r) => Region2(r),
            Region3(r) => Region3(r),
        }
    }
}

impl Import for Extra {
    fn import_from(&self, i: &Importer) -> Extra {
        if let Some(ref h) = self.h {
            let mut h2 = HashMap::with_capacity(h.len());
            for (k, v) in h.iter() {
                h2.insert(k.clone(), i.import(v));
            }
            Extra { h: Some(Box::new(h2)) }
        } else {
            Extra { h: None }
        }
    }
}
