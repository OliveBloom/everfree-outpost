//! Definition of "flattened bundles", i.e., the Bundle file format.  Everything in here is
//! `#[repr(C)]` and defined in terms of portably-sized integer types.  Furthermore, each
//! individual data structure is fixed-length - all variable-length data is stored as an index into
//! one of the top-level tables.
//!
//! The only remaining portability concern here is endianness, so we just declare the file format
//! to be little-endian, and deal with byte-order swapping later if we ever decide to port to a
//! big-endian architecture.
//!
//! ## Error Handling
//!
//! For the actual file I/O, errors are reported to the caller, and they can try to recover if they
//! want.
//!
//! If an error occurs when flattening a Bundle, we try to recover and preserve as much data as
//! possible (to allow for manual recovery).  The assumption is that this code is the caller's only
//! way of exporting the data, and any data not written here will be lost when the program shuts
//! down.
//!
//! We use the normal `Result`/`try!` error reporting when unflattening a bundle.  There is no
//! recovery mechanism.  If the data is corrupt, it needs to be fixed manually before reading it
//! in.

// TODO: error handling!

use std::collections::HashMap;
use std::io;
use std::mem;
use std::slice;
use std::str;

use types::*;
use util::Convert;
use util::{StrError, StrResult};

use world::{Motion, Item};
use world::{TerrainChunkFlags, StructureFlags};
use world::{EntityAttachment, InventoryAttachment, StructureAttachment};
use world::extra::{self, Extra};

use super::types::*;


pub type Error = StrError;
pub type Result<T> = StrResult<T>;


/// Convert usize to u32, truncating.
macro_rules! trunc32 {
    ($e:expr) => ($e as u32);
}

/// Convert usize to u32, saturating.
macro_rules! sat32 {
    ($e:expr) => (
        if $e > ::std::u32::MAX as usize {
            ::std::u32::MAX
        } else {
            $e as u32
        }
    );
}


// NB: size_of::<FileHeader> is 16, a multiple of ALIGNMENT
#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FileHeader {
    minor: u16,
    major: u16,
    header_offset: u32,
    header_count: u32,
    _reserved0: u32,
}

// NB: size_of::<SectionHeader> is 16, a multiple of ALIGNMENT
#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SectionHeader {
    tag: [u8; 4],
    offset: u32,
    count: u32,
    _reserved0: u32,
}

pub trait Section<'a> {
    type Ref;

    fn len(&self) -> usize;
    fn item_size() -> usize;

    fn new() -> Self;
    fn new_ref() -> Self::Ref;
    fn borrow(&'a self) -> Self::Ref;
    fn to_owned(r: Self::Ref) -> Self;

    unsafe fn from_bytes(buf: &'a [u8]) -> Self::Ref;
    unsafe fn as_bytes(r: Self::Ref) -> &'a [u8];

    fn byte_len(&self) -> usize {
        self.len() * Self::item_size()
    }
}

impl<'a, T: Clone> Section<'a> for Vec<T> {
    type Ref = &'a [T];

    fn len(&self) -> usize { self.len() }
    fn item_size() -> usize { mem::size_of::<T>() }

    fn new() -> Vec<T> { Vec::new() }
    fn new_ref() -> &'a [T] { &[] }
    fn borrow(&'a self) -> &'a [T] { &**self }
    fn to_owned(r: &'a [T]) -> Vec<T> { r.to_owned() }

    unsafe fn from_bytes(buf: &'a [u8]) -> &'a [T] {
        slice::from_raw_parts(buf.as_ptr() as *const T, buf.len() / mem::size_of::<T>())
    }

    unsafe fn as_bytes(r: &'a [T]) -> &'a [u8] {
        slice::from_raw_parts(r.as_ptr() as *const u8, r.len() * mem::size_of::<T>())
    }
}

impl<'a> Section<'a> for String {
    type Ref = &'a str;

    fn len(&self) -> usize { self.len() }
    fn item_size() -> usize { 1 }

    fn new() -> String { String::new() }
    fn new_ref() -> &'a str { "" }
    fn borrow(&'a self) -> &'a str { &**self }
    fn to_owned(r: &'a str) -> String { r.to_owned() }

    unsafe fn from_bytes(buf: &'a [u8]) -> &'a str {
        str::from_utf8(buf).unwrap()
    }

    unsafe fn as_bytes(r: &'a str) -> &'a [u8] {
        r.as_bytes()
    }
}

impl<'a, T: Clone> Section<'a> for Option<Box<T>> {
    type Ref = Option<&'a T>;

    fn len(&self) -> usize { if self.is_some() { 1 } else { 0 } }
    fn item_size() -> usize { mem::size_of::<T>() }

    fn new() -> Option<Box<T>> { None }
    fn new_ref() -> Option<&'a T> { None }
    fn borrow(&'a self) -> Option<&'a T> { self.as_ref().map(|x| &**x) }
    fn to_owned(r: Option<&'a T>) -> Option<Box<T>> {
        match r {
            None => None,
            Some(r) => Some(Box::new(r.clone())),
        }
    }

    unsafe fn from_bytes(buf: &'a [u8]) -> Option<&'a T> {
        if buf.len() == mem::size_of::<T>() {
            Some(&*(buf.as_ptr() as *const T))
        } else if buf.len() == 0 {
            None
        } else {
            panic!("bad buffer length");
        }
    }

    unsafe fn as_bytes(r: Option<&'a T>) -> &'a [u8] {
        match r {
            None => &[],
            Some(r) => slice::from_raw_parts(r as *const T as *const u8, mem::size_of::<T>()),
        }
    }
}

fn align(x: usize) -> usize {
    let a = ALIGNMENT - 1;
    (x + a) & !a
}

/// The maximum alignment of any field of any member of Flat.
const ALIGNMENT: usize = 8;
const CURRENT_VERSION: (u16, u16) = (1, 0);

macro_rules! filter_sections {
    (file_header, $e:expr) => { () };
    (section_headers, $e:expr) => { () };
    ($name:ident, $e:expr) => { $e };
}

macro_rules! safe_slice {
    ($arr:expr [ $range:expr ]) => {
        if $range.start > $range.end || $range.end > $arr.len() {
            fail!(concat!("slice out of bounds: ", stringify!($arr[$range])))
        } else {
            $arr[$range]
        }
    };
}

unsafe fn extract_section<'a, S: Section<'a>>(buf: &'a [u8],
                                              offset: u32,
                                              len: u32) -> Option<S::Ref> {
    let start = offset as usize;
    let end = start + len as usize * S::item_size();
    if start > end || end > buf.len() {
        None
    } else {
        Some(S::from_bytes(&buf[start .. end]))
    }
}

macro_rules! extract_section {
    ($buf:expr, $off:expr, $count:expr, $name:ident : $ty:ty) => {
        unwrap!(extract_section::<$ty>($buf, $off, $count),
                concat!("section ", stringify!($name), " extends out of bounds"))
    };
}

macro_rules! flat {
    ($($tag:expr, $name:ident : $ty:ty,)*) => {
        pub struct Flat {
            $( pub $name: $ty, )*
            pub string_map: HashMap<String, FlatStr>,
        }

        pub struct FlatView<'a> {
            $( pub $name: <$ty as Section<'a>>::Ref, )*
        }

        impl Flat {
            pub fn new() -> Flat {
                Flat {
                    $( $name: <$ty as Section>::new(), )*
                    string_map: HashMap::new(),
                }
            }

            pub fn borrow(&self) -> FlatView {
                FlatView {
                    $( $name: <$ty as Section>::borrow(&self.$name), )*
                }
            }

            fn count_sections(&self) -> usize {
                let mut count = 0;
                $(filter_sections!($name,
                    if <$ty as Section>::len(&self.$name) > 0 {
                        count += 1;
                    }
                );)*
                count
            }

            fn build_headers(&mut self) -> usize {
                // Clear file and section headers up-front.  We need to regenerate them anyway, and
                // if there are old ones around, it will confuse `count_sections`.
                self.file_header = None;
                self.section_headers = Vec::new();

                let num_sections = self.count_sections();

                self.file_header = Some(Box::new(FileHeader {
                    major: CURRENT_VERSION.0,
                    minor: CURRENT_VERSION.1,
                    header_offset: trunc32!(mem::size_of::<FileHeader>()),
                    header_count: sat32!(num_sections),
                    _reserved0: 0,
                }));

                // We set up offsets to place the sections headers at the beginning, followed by
                // each section in declaration order.
                let mut offset = mem::size_of::<FileHeader>() +
                                 mem::size_of::<SectionHeader>() * num_sections;

                $(filter_sections!($name,
                    if <$ty as Section>::len(&self.$name) > 0 {
                        self.section_headers.push(SectionHeader {
                            tag: *$tag,
                            offset: trunc32!(offset),
                            count: sat32!(<$ty as Section>::len(&self.$name)),
                            _reserved0: 0,
                        });
                        offset = align(offset + <$ty as Section>::byte_len(&self.$name));
                    }
                );)*

                offset
            }

            pub fn write<W: io::Write>(&mut self, w: &mut W) -> io::Result<()> {
                self.build_headers();

                // NB: relies on file_header and section_headers being first in declaration order.
                $(
                    {
                        let r = <$ty as Section>::borrow(&self.$name);
                        let buf = unsafe { <$ty as Section>::as_bytes(r) };
                        assert!(buf.len() == <$ty as Section>::byte_len(&self.$name));
                        let padding = (ALIGNMENT - buf.len() % ALIGNMENT) % ALIGNMENT;
                        try!(w.write_all(buf));
                        if buf.len() % ALIGNMENT != 0 {
                            // Write enough zeros to get to the next multiple of ALIGNMENT
                            try!(w.write_all(&[0; ALIGNMENT][buf.len() % ALIGNMENT ..]));
                        }
                    }
                )*

                Ok(())
            }
        }

        impl<'a> FlatView<'a> {
            fn to_owned(&self) -> Flat {
                Flat {
                    $( $name: <$ty as Section>::to_owned(self.$name), )*
                    string_map: HashMap::new(),
                }
            }

            pub fn from_bytes(buf: &'a [u8]) -> Result<FlatView<'a>> {
                let mut v = FlatView {
                    $( $name: <$ty as Section>::new_ref(), )*
                };

                if buf.as_ptr() as usize % ALIGNMENT != 0 {
                    fail!("FlatView::from_bytes: misaligned input buffer");
                }

                // extract_section returns Option<&'a ...>, but the Option is always Some.  (The
                // Option is used only to allow the field to be empty, if the FlatView is
                // constructed from a partially-initialized Flat.)
                let file_header = unsafe {
                    unwrap!(extract_section!(buf, 0, 1, file_header: Option<Box<FileHeader>>))
                };
                v.file_header = Some(file_header);

                if file_header.header_offset as usize % ALIGNMENT != 0 {
                    fail!("FlatView::from_bytes: misaligned section header offset");
                }
                let section_headers = unsafe {
                    extract_section!(buf, file_header.header_offset, file_header.header_count,
                                     section_headers: Vec<SectionHeader>)
                };
                v.section_headers = section_headers;

                for s in section_headers {
                    if s.offset as usize % ALIGNMENT != 0 {
                        fail!("FlatView::from_bytes: misaligned section offset");
                    }
                    match &s.tag {
                        $(
                            $tag => {
                                v.$name = unsafe {
                                    extract_section!(buf, s.offset, s.count, $name: $ty)
                                };
                            },
                        )*
                        _ => fail!("bad tag for section"),
                    }
                }

                Ok(v)
            }
        }
    };
}

flat! {
    // Headers
    b"HFil",  file_header: Option<Box<FileHeader>>,
    b"HSec",  section_headers: Vec<SectionHeader>,

    // Data definitions
    b"DAni",  anims: Vec<FlatStr>,
    b"DItm",  items: Vec<FlatStr>,
    b"DBlk",  blocks: Vec<FlatStr>,
    b"DTmp",  templates: Vec<FlatStr>,

    // World objects
    b"WWrl",  world: Option<Box<FlatWorld>>,
    b"WCli",  clients: Vec<FlatClient>,
    b"WEnt",  entities: Vec<FlatEntity>,
    b"WInv",  inventories: Vec<FlatInventory>,
    b"WPln",  planes: Vec<FlatPlane>,
    b"WTCk",  terrain_chunks: Vec<FlatTerrainChunk>,
    b"WStr",  structures: Vec<FlatStructure>,

    // Generic values
    b"GStr",  strings: String,
    b"GISm",  small_ints: Vec<u32>,
    b"GILg",  large_ints: Vec<u64>,
    b"GExt",  extras: Vec<FlatExtra>,
    b"GHsh",  hash_entries: Vec<FlatEntry>,

    // World object components
    b"CItm",  inv_items: Vec<CItem>,
    b"CLdC",  loaded_chunks: Vec<CLoadedChunk>,
    b"CBlC",  block_chunks: Vec<CBlockChunk>,
}



// There are two categories of types in here.  "FlatX" is an "X" that has been packed into the
// `Flat` object, using offset+length to refer to variable-size data in the `Flat` arrays. "CX" is
// just an "X" that has been converted to a `#[repr(C)]` format.

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CV2 {
    pub x: i32,
    pub y: i32,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CV3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CAttachment {
    pub tag: u8,
    pub data: u32,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CLoadedChunk {
    pub cpos: CV2,
    pub tcid: u64,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CItem {
    pub tag: u8,
    pub extra: u8,
    pub id: u16,
}

#[repr(C)]
pub struct CBlockChunk {
    pub data: BlockChunk,
}
// [T; 4096] is Copy but not Clone, so we need to impl Clone manually.
impl Clone for CBlockChunk {
    #[inline]
    fn clone(&self) -> CBlockChunk {
        CBlockChunk { data: self.data }
    }
}


#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlatStr {
    pub off: u32,
    pub len: u32,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlatVec {
    pub off: u32,
    pub len: u32,
}

// `Extra` is a little complicated.  Sometimes the `data` field is a `Flat` array offset, but other
// times it's an immediate value.
#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlatExtra {
    pub tag: u8,
    pub a: u8,
    pub b: u16,
    pub data: u32,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlatEntry {
    pub key: FlatStr,
    pub value: FlatExtra,
}

#[repr(C)] #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlatBlockChunk {
    pub off: u32,
}


#[repr(C)] #[derive(Clone)]
pub struct FlatWorld {
    pub now: i64,

    pub next_client: u64,
    pub next_entity: u64,
    pub next_inventory: u64,
    pub next_plane: u64,
    pub next_terrain_chunk: u64,
    pub next_structure: u64,

    pub extra: FlatExtra,
    pub child_entities: FlatVec,
    pub child_inventories: FlatVec,
}

#[repr(C)] #[derive(Clone)]
pub struct FlatClient {
    pub name: FlatStr,
    pub pawn: u32,

    pub extra: FlatExtra,
    pub stable_id: u64,
    pub child_entities: FlatVec,
    pub child_inventories: FlatVec,
}

#[repr(C)] #[derive(Clone)]
pub struct FlatEntity {
    pub stable_plane: u64,

    pub motion_start_time: i64,
    pub motion_duration: u16,
    pub motion_start_pos: CV3,
    pub motion_end_pos: CV3,

    pub anim: u16,
    pub facing: CV3,
    pub target_velocity: CV3,
    pub appearance: u32,

    pub extra: FlatExtra,
    pub stable_id: u64,
    pub attachment: CAttachment,
    pub child_inventories: FlatVec,
}

#[repr(C)] #[derive(Clone)]
pub struct FlatInventory {
    pub contents: FlatVec,

    pub extra: FlatExtra,
    pub stable_id: u64,
    pub attachment: CAttachment,
}

#[repr(C)] #[derive(Clone)]
pub struct FlatPlane {
    pub name: FlatStr,

    pub saved_chunks: FlatVec,

    pub extra: FlatExtra,
    pub stable_id: u64,
    pub child_structures: FlatVec,
}

#[repr(C)] #[derive(Clone)]
pub struct FlatTerrainChunk {
    pub stable_plane: u64,
    pub cpos: CV2,
    pub blocks: FlatBlockChunk,

    pub extra: FlatExtra,
    pub stable_id: u64,
    pub flags: u32,
    pub child_structures: FlatVec,
}

#[repr(C)] #[derive(Clone)]
pub struct FlatStructure {
    pub stable_plane: u64,
    pub pos: CV3,
    pub template: u32,

    pub extra: FlatExtra,
    pub stable_id: u64,
    pub flags: u32,
    pub attachment: CAttachment,
    pub child_inventories: FlatVec,
}


//
// Traits
//

// Elements of type `Flatten` can be packed down to a single index into some `Flat` array.
trait Flatten: Sized {
    /// Pack an object into the appropriate `Flat` array and return its offset in the primary array
    /// for this type.
    fn flatten_idx(&self, f: &mut Flat) -> usize;

    /// Unpack an object from the `Flat` arrays at the given offset into the primary array.
    fn unflatten_idx(off: usize, f: &FlatView) -> Self;

    fn flatten(&self, f: &mut Flat) -> u32 {
        Flatten::flatten_idx(self, f).to_u32().unwrap()
    }
    fn unflatten(off: u32, f: &FlatView) -> Self {
        Flatten::unflatten_idx(off as usize, f)
    }
}

trait FixedSize: Flatten {
    /// Number of slots taken up in the primary array by each item of this type.
    fn size() -> usize { 1 }
}

trait FlattenPart {
    type Part;
    /// Pack all elements or other sub-components of the object, and return a representation that
    /// is safe to store inside other objects.
    fn flatten_part(&self, f: &mut Flat) -> Self::Part;
}

trait UnflattenPart<'a> {
    type Part;
    fn unflatten_part(part: &Self::Part, f: &FlatView<'a>) -> Self;
}

trait Conv: Copy {
    type C: Copy;
    fn conv(self) -> Self::C;
    fn unconv(Self::C) -> Self;
}


//
// Wrapper functions for using the trait methods above
//

impl Flat {
    fn flatten<T: ?Sized + Flatten>(&mut self, x: &T) -> u32 {
        Flatten::flatten(x, self)
    }

    fn flatten_part<T: ?Sized + FlattenPart>(&mut self, x: &T) -> T::Part {
        FlattenPart::flatten_part(x, self)
    }

    pub fn flatten_bundle(&mut self, b: &Bundle) {
        self.flatten(b);
    }
}

impl<'a> FlatView<'a> {
    fn unflatten<T: Flatten>(&self, off: u32) -> T {
        Flatten::unflatten(off, self)
    }

    fn unflatten_part<T: UnflattenPart<'a>>(&self, obj: &T::Part) -> T {
        UnflattenPart::unflatten_part(obj, self)
    }

    pub fn unflatten_bundle(&self) -> Bundle {
        self.unflatten(0)
    }
}

fn conv<T: Conv>(x: T) -> T::C {
    Conv::conv(x)
}

fn unconv<T: Conv>(obj: T::C) -> T {
    Conv::unconv(obj)
}


//
// Simple types
//

impl Flatten for u32 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.small_ints.len();
        f.small_ints.push(*self);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> u32 {
        f.small_ints[off]
    }
}
impl FixedSize for u32 {}

impl Flatten for i32 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        <u32 as Flatten>::flatten_idx(&(*self as u32), f)
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> i32 {
        <u32 as Flatten>::unflatten_idx(off, f) as i32
    }
}
impl FixedSize for i32 {}

impl Flatten for u64 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.large_ints.len();
        f.large_ints.push(*self);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> u64 {
        f.large_ints[off]
    }
}
impl FixedSize for u64 {}

impl Flatten for i64 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        <u64 as Flatten>::flatten_idx(&(*self as u64), f)
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> i64 {
        <u64 as Flatten>::unflatten_idx(off, f) as i64
    }
}
impl FixedSize for i64 {}

impl Flatten for f64 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let x = unsafe { mem::transmute::<f64, u64>(*self) };
        Flatten::flatten_idx(&x, f)
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> f64 {
        let x = Flatten::unflatten_idx(off, f);
        unsafe { mem::transmute::<u64, f64>(x) }
    }
}
impl FixedSize for f64 {}


// TODO: Copy restriction shouldn't be needed, but the derived Copy instance for Stable wants it
impl<T: Copy> Flatten for Stable<T> {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        <u64 as Flatten>::flatten_idx(&self.unwrap(), f)
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Stable<T> {
        Stable::new(<u64 as Flatten>::unflatten_idx(off, f))
    }
}
impl<T: Copy> FixedSize for Stable<T> {}

impl<T: Copy> Conv for Stable<T> {
    type C = u64;
    fn conv(self) -> u64 {
        self.unwrap()
    }
    fn unconv(obj: u64) -> Stable<T> {
        Stable::new(obj)
    }
}


macro_rules! id_conv {
    ($ObjId:ident) => { id_conv!($ObjId, u32); };
    ($ObjId:ident, $raw:ty) => {
        impl Conv for $ObjId {
            type C = u32;
            fn conv(self) -> u32 {
                self.unwrap() as u32
            }
            fn unconv(obj: u32) -> $ObjId {
                $ObjId(obj as $raw)
            }
        }

        impl Conv for Option<$ObjId> {
            type C = u32;
            fn conv(self) -> u32 {
                match self {
                    Some(x) => x.unwrap() as u32,
                    None => -1_i32 as u32
                }
            }
            fn unconv(obj: u32) -> Option<$ObjId> {
                if obj == -1_i32 as u32 {
                    None
                } else {
                    Some($ObjId(obj as $raw))
                }
            }
        }

        impl Flatten for $ObjId {
            fn flatten_idx(&self, f: &mut Flat) -> usize {
                Flatten::flatten_idx(&conv(*self), f)
            }
            fn unflatten_idx(off: usize, f: &FlatView) -> $ObjId {
                unconv(Flatten::unflatten_idx(off, f))
            }
        }
        impl FixedSize for $ObjId {}
    };
}

id_conv!(ClientId, u16);
id_conv!(EntityId);
id_conv!(InventoryId);
id_conv!(PlaneId);
id_conv!(TerrainChunkId);
id_conv!(StructureId);


impl Flatten for V2 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.small_ints.len();
        f.small_ints.push(self.x as u32);
        f.small_ints.push(self.y as u32);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> V2 {
        V2 {
            x: f.small_ints[off + 0] as i32,
            y: f.small_ints[off + 1] as i32,
        }
    }
}

impl FixedSize for V2 {
    fn size() -> usize { 2 }
}

impl Conv for V2 {
    type C = CV2;

    fn conv(self) -> CV2 {
        CV2 { x: self.x, y: self.y }
    }

    fn unconv(obj: CV2) -> V2 {
        V2 { x: obj.x, y: obj.y }
    }
}


impl Flatten for V3 {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.small_ints.len();
        f.small_ints.push(self.x as u32);
        f.small_ints.push(self.y as u32);
        f.small_ints.push(self.z as u32);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> V3 {
        V3 {
            x: f.small_ints[off + 0] as i32,
            y: f.small_ints[off + 1] as i32,
            z: f.small_ints[off + 2] as i32,
        }
    }
}

impl FixedSize for V3 {
    fn size() -> usize { 3 }
}

impl Conv for V3 {
    type C = CV3;

    fn conv(self) -> CV3 {
        CV3 { x: self.x, y: self.y, z: self.z }
    }

    fn unconv(obj: CV3) -> V3 {
        V3 { x: obj.x, y: obj.y, z: obj.z }
    }
}


impl<V: Vn + Flatten + FixedSize> Flatten for Region<V> {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off1 = Flatten::flatten_idx(&self.min, f);
        let off2 = Flatten::flatten_idx(&self.max, f);
        assert!(off2 - off1 == <V as FixedSize>::size());
        off1
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Region<V> {
        let min = Flatten::unflatten_idx(off, f);
        let max = Flatten::unflatten_idx(off + <V as FixedSize>::size(), f);
        Region::new(min, max)
    }
}


impl Conv for EntityAttachment {
    type C = CAttachment;

    fn conv(self) -> CAttachment {
        match self {
            EntityAttachment::World => CAttachment { tag: 0, data: 0 },
            EntityAttachment::Chunk => CAttachment { tag: 1, data: 0 },
            EntityAttachment::Client(id) => CAttachment { tag: 2, data: conv(id) },
        }
    }

    fn unconv(obj: CAttachment) -> EntityAttachment {
        match obj.tag {
            0 => EntityAttachment::World,
            1 => EntityAttachment::Chunk,
            2 => EntityAttachment::Client(unconv(obj.data)),
            _ => panic!("bad tag for EntityAttachment"),
        }
    }
}

impl Conv for InventoryAttachment {
    type C = CAttachment;

    fn conv(self) -> CAttachment {
        match self {
            InventoryAttachment::World => CAttachment { tag: 0, data: 0 },
            InventoryAttachment::Client(id) => CAttachment { tag: 1, data: conv(id) },
            InventoryAttachment::Entity(id) => CAttachment { tag: 2, data: conv(id) },
            InventoryAttachment::Structure(id) => CAttachment { tag: 3, data: conv(id) },
        }
    }

    fn unconv(obj: CAttachment) -> InventoryAttachment {
        match obj.tag {
            0 => InventoryAttachment::World,
            1 => InventoryAttachment::Client(unconv(obj.data)),
            2 => InventoryAttachment::Entity(unconv(obj.data)),
            3 => InventoryAttachment::Structure(unconv(obj.data)),
            _ => panic!("bad tag for InventoryAttachment"),
        }
    }
}

impl Conv for StructureAttachment {
    type C = CAttachment;

    fn conv(self) -> CAttachment {
        match self {
            StructureAttachment::Plane => CAttachment { tag: 0, data: 0 },
            StructureAttachment::Chunk => CAttachment { tag: 1, data: 0 },
        }
    }

    fn unconv(obj: CAttachment) -> StructureAttachment {
        match obj.tag {
            0 => StructureAttachment::Plane,
            1 => StructureAttachment::Chunk,
            _ => panic!("bad tag for StructureAttachment"),
        }
    }
}


impl Conv for (V2, Stable<TerrainChunkId>) {
    type C = CLoadedChunk;

    fn conv(self) -> CLoadedChunk {
        let (cpos, tcid) = self;
        CLoadedChunk {
            cpos: conv(cpos),
            tcid: conv(tcid),
        }
    }

    fn unconv(obj: CLoadedChunk) -> (V2, Stable<TerrainChunkId>) {
        (unconv(obj.cpos), unconv(obj.tcid))
    }
}

impl Flatten for (V2, Stable<TerrainChunkId>) {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.loaded_chunks.len();
        f.loaded_chunks.push(Conv::conv(*self));
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> (V2, Stable<TerrainChunkId>) {
        Conv::unconv(f.loaded_chunks[off])
    }
}

impl FixedSize for (V2, Stable<TerrainChunkId>) {}


impl Conv for Item {
    type C = CItem;

    fn conv(self) -> CItem {
        match self {
            Item::Empty => CItem { tag: 0, extra: 0, id: 0 },
            Item::Bulk(count, id) => CItem { tag: 1, extra: count, id: id },
            Item::Special(extra, id) => CItem { tag: 1, extra: extra, id: id },
        }
    }

    fn unconv(obj: CItem) -> Item {
        match obj.tag {
            0 => Item::Empty,
            1 => Item::Bulk(obj.extra, obj.id),
            2 => Item::Special(obj.extra, obj.id),
            _ => panic!("bad tag for Item"),
        }
    }
}

impl Flatten for Item {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.inv_items.len();
        f.inv_items.push(conv(*self));
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Item {
        unconv(f.inv_items[off])
    }
}

impl FixedSize for Item {}


impl Conv for TerrainChunkFlags {
    type C = u32;
    fn conv(self) -> u32 {
        self.bits()
    }
    fn unconv(obj: u32) -> TerrainChunkFlags {
        TerrainChunkFlags::from_bits(obj).unwrap()
    }
}

impl Conv for StructureFlags {
    type C = u32;
    fn conv(self) -> u32 {
        self.bits()
    }
    fn unconv(obj: u32) -> StructureFlags {
        StructureFlags::from_bits(obj).unwrap()
    }
}


//
// Collections
//

impl FlattenPart for str {
    type Part = FlatStr;
    fn flatten_part(&self, f: &mut Flat) -> FlatStr {
        if let Some(&fstr) = f.string_map.get(self) {
            return fstr;
        }

        let fstr = FlatStr {
            off: f.strings.len().to_u32().unwrap(),
            len: self.len().to_u32().unwrap(),
        };
        f.strings.push_str(self);
        f.string_map.insert(self.to_owned(), fstr);
        fstr
    }
}

impl FlattenPart for Box<str> {
    type Part = FlatStr;
    fn flatten_part(&self, f: &mut Flat) -> FlatStr {
        FlattenPart::flatten_part(&**self, f)
    }
}

impl<'a> UnflattenPart<'a> for &'a str {
    type Part = FlatStr;
    fn unflatten_part(obj: &FlatStr, f: &FlatView<'a>) -> &'a str {
        let off = obj.off.to_usize().unwrap();
        let len = obj.len.to_usize().unwrap();
        &f.strings[off .. off + len]
    }
}

impl<'a> UnflattenPart<'a> for String {
    type Part = FlatStr;
    fn unflatten_part(obj: &FlatStr, f: &FlatView<'a>) -> String {
        f.unflatten_part::<&str>(obj).to_owned()
    }
}

impl<'a> UnflattenPart<'a> for Box<str> {
    type Part = FlatStr;
    fn unflatten_part(obj: &FlatStr, f: &FlatView<'a>) -> Box<str> {
        f.unflatten_part::<String>(obj).into_boxed_slice()
    }
}


impl<T: Flatten + FixedSize> FlattenPart for [T] {
    type Part = FlatVec;
    fn flatten_part(&self, f: &mut Flat) -> FlatVec {
        if self.len() == 0 {
            return FlatVec { off: 0, len: 0 }
        }

        let off = self[0].flatten_idx(f);
        let mut last = off;
        for x in &self[1..] {
            let cur = x.flatten_idx(f);
            assert!(cur == last + <T as FixedSize>::size());
            last = cur;
        }
        FlatVec {
            off: off.to_u32().unwrap(),
            len: self.len().to_u32().unwrap(),
        }
    }
}

impl<T: Flatten + FixedSize> FlattenPart for Box<[T]> {
    type Part = FlatVec;
    fn flatten_part(&self, f: &mut Flat) -> FlatVec {
        f.flatten_part(&**self)
    }
}

impl<'a, T: Flatten + FixedSize> UnflattenPart<'a> for Vec<T> {
    type Part = FlatVec;
    fn unflatten_part(obj: &FlatVec, f: &FlatView<'a>) -> Vec<T> {
        let base = obj.off.to_usize().unwrap();
        let step = <T as FixedSize>::size();
        let len = obj.len.to_usize().unwrap();

        let mut v = Vec::with_capacity(len);
        for i in 0 .. obj.len.to_usize().unwrap() {
            let off = base + i * step;
            v.push(Flatten::unflatten_idx(off, f));
        }
        v
    }
}

impl<'a, T: Flatten + FixedSize> UnflattenPart<'a> for Box<[T]> {
    type Part = FlatVec;
    fn unflatten_part(obj: &FlatVec, f: &FlatView<'a>) -> Box<[T]> {
        f.unflatten_part::<Vec<T>>(obj).into_boxed_slice()
    }
}


//
// Extra
//

// Common defs

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
        Int =           0x02,
        Float =         0x03,
        Str =           0x04,

        Array =         0x10,
        Hash =          0x11,

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

impl Flatten for FlatExtra {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.extras.len();
        f.extras.push(*self);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> FlatExtra {
        f.extras[off]
    }
}
impl FixedSize for FlatExtra {}

impl Flatten for FlatEntry {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.hash_entries.len();
        f.hash_entries.push(*self);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> FlatEntry {
        f.hash_entries[off]
    }
}
impl FixedSize for FlatEntry {}

// Flatten

impl FlatExtra {
    fn new(tag: Tag, a: u8, b: u16, data: u32) -> FlatExtra {
        FlatExtra {
            tag: tag as u8,
            a: a,
            b: b,
            data: data,
        }
    }

    fn from_tag(tag: Tag) -> FlatExtra {
        FlatExtra::new(tag, 0, 0, 0)
    }

    fn from_data(tag: Tag, data: u32) -> FlatExtra {
        FlatExtra::new(tag, 0, 0, data)
    }

    fn from_off_len(tag: Tag, off: u32, len: u32) -> FlatExtra {
        FlatExtra::new(tag, 0, len.to_u16().unwrap(), off)
    }
}

fn flatten_view(x: &extra::View, f: &mut Flat) -> FlatExtra {
    match *x {
        extra::View::Value(ref v) => flatten_value(v, f),
        extra::View::Array(ref a) => flatten_array(a.iter(), f),
        extra::View::Hash(ref h) => flatten_hash(h.iter(), f),
    }
}

fn flatten_value(x: &extra::Value, f: &mut Flat) -> FlatExtra {
    use world::extra::Value;
    match *x {
        Value::Null => FlatExtra::from_tag(Tag::Null),
        Value::Bool(b) => FlatExtra::from_data(Tag::Bool, b as u32),
        Value::Int(i) => {
            if let Some(small_i) = i.to_i32() {
                FlatExtra::new(Tag::Int, 1, 0, small_i as u32)
            } else {
                FlatExtra::new(Tag::Int, 0, 0, f.flatten(&i))
            }
        },
        Value::Float(x) => FlatExtra::from_data(Tag::Float, f.flatten(&x)),
        Value::Str(ref s) => {
            let fs = f.flatten_part(s as &str);
            FlatExtra::from_off_len(Tag::Str, fs.off, fs.len)
        },

        Value::ClientId(id) => FlatExtra::from_data(Tag::ClientId, conv(id)),
        Value::EntityId(id) => FlatExtra::from_data(Tag::EntityId, conv(id)),
        Value::InventoryId(id) => FlatExtra::from_data(Tag::InventoryId, conv(id)),
        Value::PlaneId(id) => FlatExtra::from_data(Tag::PlaneId, conv(id)),
        Value::TerrainChunkId(id) => FlatExtra::from_data(Tag::TerrainChunkId, conv(id)),
        Value::StructureId(id) => FlatExtra::from_data(Tag::StructureId, conv(id)),

        Value::StableClientId(id) =>
            FlatExtra::from_data(Tag::StableClientId, f.flatten(&id)),
        Value::StableEntityId(id) =>
            FlatExtra::from_data(Tag::StableEntityId, f.flatten(&id)),
        Value::StableInventoryId(id) =>
            FlatExtra::from_data(Tag::StableInventoryId, f.flatten(&id)),
        Value::StablePlaneId(id) =>
            FlatExtra::from_data(Tag::StablePlaneId, f.flatten(&id)),
        Value::StableTerrainChunkId(id) =>
            FlatExtra::from_data(Tag::StableTerrainChunkId, f.flatten(&id)),
        Value::StableStructureId(id) =>
            FlatExtra::from_data(Tag::StableStructureId, f.flatten(&id)),

        Value::V2(v2) => FlatExtra::from_data(Tag::V2, f.flatten(&v2)),
        Value::V3(v3) => FlatExtra::from_data(Tag::V3, f.flatten(&v3)),
        Value::Region2(r2) => FlatExtra::from_data(Tag::Region2, f.flatten(&r2)),
        Value::Region3(r3) => FlatExtra::from_data(Tag::Region3, f.flatten(&r3)),
    }
}

fn flatten_array<'a, I>(mut i: I, f: &mut Flat) -> FlatExtra
        where I: Iterator<Item=extra::View<'a>> {
    let mut items = Vec::with_capacity(i.size_hint().0);
    for x in i {
        let fx = flatten_view(&x, f);
        items.push(fx);
    }
    let flat_items = f.flatten_part(&items as &[FlatExtra]);
    FlatExtra::from_off_len(Tag::Array, flat_items.off, flat_items.len)
}

fn flatten_hash<'a, I>(mut i: I, f: &mut Flat) -> FlatExtra
        where I: Iterator<Item=(&'a str, extra::View<'a>)> {
    let mut items = Vec::with_capacity(i.size_hint().0);
    for (k, v) in i {
        let fk = f.flatten_part(k);
        let fv = flatten_view(&v, f);
        items.push(FlatEntry { key: fk, value: fv });
    }
    let flat_items = f.flatten_part(&items as &[FlatEntry]);
    FlatExtra::from_off_len(Tag::Hash, flat_items.off, flat_items.len)
}

// Unflatten

fn unflatten_entries<'a>(f: &FlatView<'a>, off: u32, len: u16) -> &'a [FlatEntry] {
    let off = off.to_usize().unwrap();
    let len = len.to_usize().unwrap();
    &f.hash_entries[off .. off + len]
}

fn unflatten_extras<'a>(f: &FlatView<'a>, off: u32, len: u16) -> &'a [FlatExtra] {
    let off = off.to_usize().unwrap();
    let len = len.to_usize().unwrap();
    &f.extras[off .. off + len]
}

fn unflatten_hash(fx: &FlatExtra, f: &FlatView, mut v: extra::HashViewMut) {
    assert!(fx.tag == Tag::Hash as u8);
    for e in unflatten_entries(f, fx.data, fx.b) {
        let key = <String as UnflattenPart>::unflatten_part(&e.key, f);
        match Tag::from_primitive(e.value.tag).unwrap() {
            Tag::Hash => unflatten_hash(&e.value, f, v.borrow().set_hash(&key)),
            Tag::Array => unflatten_array(&e.value, f, v.borrow().set_array(&key)),
            _ => v.borrow().set(&key, unflatten_value(&e.value, f)),
        }
    }
}

fn unflatten_array(fx: &FlatExtra, f: &FlatView, mut v: extra::ArrayViewMut) {
    assert!(fx.tag == Tag::Array as u8);
    assert!(v.borrow().len() == 0);
    for (i, e) in unflatten_extras(f, fx.data, fx.b).iter().enumerate() {
        v.borrow().push();
        match Tag::from_primitive(e.tag).unwrap() {
            Tag::Hash => unflatten_hash(e, f, v.borrow().set_hash(i)),
            Tag::Array => unflatten_array(e, f, v.borrow().set_array(i)),
            _ => v.borrow().set(i, unflatten_value(e, f)),
        }
    }
}

fn unflatten_value(fx: &FlatExtra, f: &FlatView) -> extra::Value {
    use world::extra::Value;
    match Tag::from_primitive(fx.tag).unwrap() {
        Tag::Null => Value::Null,
        Tag::Bool => Value::Bool(fx.data != 0),
        Tag::Int => {
            if fx.a != 0 {
                // Data is store inline
                Value::Int(fx.data as i32 as i64)
            } else {
                // Data stored in large_ints array
                Value::Int(f.unflatten(fx.data))
            }
        },
        Tag::Float => Value::Float(f.unflatten(fx.data)),
        Tag::Str => {
            let fs = FlatStr { off: fx.data, len: fx.b as u32 };
            Value::Str(f.unflatten_part(&fs))
        },

        Tag::Hash => panic!("unexpected Tag::Hash in unflatten_value"),
        Tag::Array => panic!("unexpected Tag::Array in unflatten_value"),

        Tag::ClientId => Value::ClientId(unconv(fx.data)),
        Tag::EntityId => Value::EntityId(unconv(fx.data)),
        Tag::InventoryId => Value::InventoryId(unconv(fx.data)),
        Tag::PlaneId => Value::PlaneId(unconv(fx.data)),
        Tag::TerrainChunkId => Value::TerrainChunkId(unconv(fx.data)),
        Tag::StructureId => Value::StructureId(unconv(fx.data)),

        Tag::StableClientId => Value::StableClientId(f.unflatten(fx.data)),
        Tag::StableEntityId => Value::StableEntityId(f.unflatten(fx.data)),
        Tag::StableInventoryId => Value::StableInventoryId(f.unflatten(fx.data)),
        Tag::StablePlaneId => Value::StablePlaneId(f.unflatten(fx.data)),
        Tag::StableTerrainChunkId => Value::StableTerrainChunkId(f.unflatten(fx.data)),
        Tag::StableStructureId => Value::StableStructureId(f.unflatten(fx.data)),

        Tag::V2 => Value::V2(f.unflatten(fx.data)),
        Tag::V3 => Value::V3(f.unflatten(fx.data)),
        Tag::Region2 => Value::Region2(f.unflatten(fx.data)),
        Tag::Region3 => Value::Region3(f.unflatten(fx.data)),
    }
}

// Impls

impl FlattenPart for Extra {
    type Part = FlatExtra;
    fn flatten_part(&self, f: &mut Flat) -> FlatExtra {
        if self.len() == 0 {
            return FlatExtra { tag: Tag::Null as u8, a: 0, b: 0, data: 0 };
        }

        flatten_hash(self.iter(), f)
    }
}

impl<'a> UnflattenPart<'a> for Extra {
    type Part = FlatExtra;
    fn unflatten_part(obj: &FlatExtra, f: &FlatView) -> Extra {
        let mut extra = Extra::new();
        let tag = Tag::from_primitive(obj.tag).unwrap();
        match tag {
            Tag::Null => extra,
            Tag::Hash => {
                for e in unflatten_entries(f, obj.data, obj.b) {
                    let key = <String as UnflattenPart>::unflatten_part(&e.key, f);
                    match Tag::from_primitive(e.value.tag).unwrap() {
                        Tag::Hash => unflatten_hash(&e.value, f, extra.set_hash(&key)),
                        Tag::Array => unflatten_array(&e.value, f, extra.set_array(&key)),
                        _ => extra.set(&key, unflatten_value(&e.value, f)),
                    }
                }
                extra
            },
            _ => panic!("unsupported tag for Extra"),
        }
    }
}


//
// Misc
//

impl FlattenPart for Box<BlockChunk> {
    type Part = FlatBlockChunk;
    fn flatten_part(&self, f: &mut Flat) -> FlatBlockChunk {
        let off = f.block_chunks.len();
        f.block_chunks.push(CBlockChunk { data: **self });
        FlatBlockChunk { off: off.to_u32().unwrap() }
    }
}

impl<'a> UnflattenPart<'a> for Box<BlockChunk> {
    type Part = FlatBlockChunk;
    fn unflatten_part(obj: &FlatBlockChunk, f: &FlatView) -> Box<BlockChunk> {
        Box::new(f.block_chunks[obj.off as usize].data)
    }
}


//
// Bundle Objects
//

impl Flatten for World {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let fw = FlatWorld {
            now: self.now,

            next_client: self.next_client,
            next_entity: self.next_entity,
            next_inventory: self.next_inventory,
            next_plane: self.next_plane,
            next_terrain_chunk: self.next_terrain_chunk,
            next_structure: self.next_structure,

            extra: f.flatten_part(&self.extra),
            child_entities: f.flatten_part(&self.child_entities),
            child_inventories: f.flatten_part(&self.child_inventories),
        };
        f.world = Some(Box::new(fw));
        0
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> World {
        assert!(off == 0);
        let fw = f.world.as_ref().unwrap() as &FlatWorld;
        World {
            now: fw.now,

            next_client: fw.next_client,
            next_entity: fw.next_entity,
            next_inventory: fw.next_inventory,
            next_plane: fw.next_plane,
            next_terrain_chunk: fw.next_terrain_chunk,
            next_structure: fw.next_structure,

            extra: f.unflatten_part(&fw.extra),
            child_entities: f.unflatten_part(&fw.child_entities),
            child_inventories: f.unflatten_part(&fw.child_inventories),
        }
    }
}

impl Flatten for Client {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.clients.len();
        let fc = FlatClient {
            name: f.flatten_part(&self.name as &str),
            pawn: conv(self.pawn),

            extra: f.flatten_part(&self.extra),
            stable_id: self.stable_id,
            child_entities: f.flatten_part(&self.child_entities),
            child_inventories: f.flatten_part(&self.child_inventories),
        };
        f.clients.push(fc);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Client {
        let fc = &f.clients[off];
        Client {
            name: f.unflatten_part(&fc.name),
            pawn: unconv(fc.pawn),

            extra: f.unflatten_part(&fc.extra),
            stable_id: fc.stable_id,
            child_entities: f.unflatten_part(&fc.child_entities),
            child_inventories: f.unflatten_part(&fc.child_inventories),
        }
    }
}

impl Flatten for Entity {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.entities.len();
        let fe = FlatEntity {
            stable_plane: conv(self.stable_plane),

            motion_start_time: self.motion.start_time,
            motion_duration: self.motion.duration,
            motion_start_pos: conv(self.motion.start_pos),
            motion_end_pos: conv(self.motion.end_pos),

            anim: self.anim,
            facing: conv(self.facing),
            target_velocity: conv(self.target_velocity),
            appearance: self.appearance,

            extra: f.flatten_part(&self.extra),
            stable_id: self.stable_id,
            attachment: conv(self.attachment),
            child_inventories: f.flatten_part(&self.child_inventories),
        };
        f.entities.push(fe);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Entity {
        let fe = &f.entities[off];
        Entity {
            stable_plane: unconv(fe.stable_plane),

            motion: Motion {
                start_time: fe.motion_start_time,
                duration: fe.motion_duration,
                start_pos: unconv(fe.motion_start_pos),
                end_pos: unconv(fe.motion_end_pos),
            },

            anim: fe.anim,
            facing: unconv(fe.facing),
            target_velocity: unconv(fe.target_velocity),
            appearance: fe.appearance,

            extra: f.unflatten_part(&fe.extra),
            stable_id: fe.stable_id,
            attachment: unconv(fe.attachment),
            child_inventories: f.unflatten_part(&fe.child_inventories),
        }
    }
}

impl Flatten for Inventory {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.inventories.len();
        let fi = FlatInventory {
            contents: f.flatten_part(&self.contents),

            extra: f.flatten_part(&self.extra),
            stable_id: self.stable_id,
            attachment: conv(self.attachment),
        };
        f.inventories.push(fi);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Inventory {
        let fi = &f.inventories[off];
        Inventory {
            contents: f.unflatten_part(&fi.contents),

            extra: f.unflatten_part(&fi.extra),
            stable_id: fi.stable_id,
            attachment: unconv(fi.attachment),
        }
    }
}

impl Flatten for Plane {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.planes.len();
        let fp = FlatPlane {
            name: f.flatten_part(&self.name as &str),

            saved_chunks: f.flatten_part(&self.saved_chunks),

            extra: f.flatten_part(&self.extra),
            stable_id: self.stable_id,
            child_structures: f.flatten_part(&[] as &[StructureId]),    // TODO
        };
        f.planes.push(fp);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Plane {
        let fp = &f.planes[off];
        Plane {
            name: f.unflatten_part(&fp.name),

            saved_chunks: f.unflatten_part(&fp.saved_chunks),

            extra: f.unflatten_part(&fp.extra),
            stable_id: fp.stable_id,
            //child_structures: f.unflatten_part(&fp.child_structures),
        }
    }
}

impl Flatten for TerrainChunk {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.terrain_chunks.len();
        let ftc = FlatTerrainChunk {
            stable_plane: conv(self.stable_plane),
            cpos: conv(self.cpos),
            blocks: f.flatten_part(&self.blocks),

            extra: f.flatten_part(&self.extra),
            stable_id: self.stable_id,
            flags: conv(self.flags),
            child_structures: f.flatten_part(&self.child_structures),
        };
        f.terrain_chunks.push(ftc);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> TerrainChunk {
        let ftc = &f.terrain_chunks[off];
        TerrainChunk {
            stable_plane: unconv(ftc.stable_plane),
            cpos: unconv(ftc.cpos),
            blocks: f.unflatten_part(&ftc.blocks),

            extra: f.unflatten_part(&ftc.extra),
            stable_id: ftc.stable_id,
            flags: unconv(ftc.flags),
            child_structures: f.unflatten_part(&ftc.child_structures),
        }
    }
}

impl Flatten for Structure {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        let off = f.structures.len();
        let fs = FlatStructure {
            stable_plane: conv(self.stable_plane),
            pos: conv(self.pos),
            template: self.template,

            extra: f.flatten_part(&self.extra),
            stable_id: self.stable_id,
            flags: conv(self.flags),
            attachment: conv(self.attachment),
            child_inventories: f.flatten_part(&self.child_inventories),
        };
        f.structures.push(fs);
        off
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Structure {
        let fs = &f.structures[off];
        Structure {
            stable_plane: unconv(fs.stable_plane),
            pos: unconv(fs.pos),
            template: fs.template,

            extra: f.unflatten_part(&fs.extra),
            stable_id: fs.stable_id,
            flags: unconv(fs.flags),
            attachment: unconv(fs.attachment),
            child_inventories: f.unflatten_part(&fs.child_inventories),
        }
    }
}

impl Flatten for Bundle {
    fn flatten_idx(&self, f: &mut Flat) -> usize {
        for s in &*self.anims {
            let fs = f.flatten_part(s);
            f.anims.push(fs);
        }
        for s in &*self.items {
            let fs = f.flatten_part(s);
            f.items.push(fs);
        }
        for s in &*self.blocks {
            let fs = f.flatten_part(s);
            f.blocks.push(fs);
        }
        for s in &*self.templates {
            let fs = f.flatten_part(s);
            f.templates.push(fs);
        }

        if let Some(ref w) = self.world {
            f.flatten(w as &World);
        }
        for c in &*self.clients {
            f.flatten(c);
        }
        for e in &*self.entities {
            f.flatten(e);
        }
        for i in &*self.inventories {
            f.flatten(i);
        }
        for p in &*self.planes {
            f.flatten(p);
        }
        for tc in &*self.terrain_chunks {
            f.flatten(tc);
        }
        for s in &*self.structures {
            f.flatten(s);
        }

        0
    }

    fn unflatten_idx(off: usize, f: &FlatView) -> Bundle {
        assert!(off == 0);

        Bundle {
            anims: unflatten_strs(f, &f.anims),
            items: unflatten_strs(f, &f.items),
            blocks: unflatten_strs(f, &f.blocks),
            templates: unflatten_strs(f, &f.templates),

            world: if f.world.is_some() { Some(Box::new(f.unflatten(0))) } else { None },
            clients: unflatten_idxs(f, f.clients.len()),
            entities: unflatten_idxs(f, f.entities.len()),
            inventories: unflatten_idxs(f, f.inventories.len()),
            planes: unflatten_idxs(f, f.planes.len()),
            terrain_chunks: unflatten_idxs(f, f.terrain_chunks.len()),
            structures: unflatten_idxs(f, f.structures.len()),
        }
    }
}

fn unflatten_strs(f: &FlatView, fs: &[FlatStr]) -> Box<[Box<str>]> {
    fs.iter().map(|fs| f.unflatten_part(fs))
      .collect::<Vec<_>>().into_boxed_slice()
}

fn unflatten_idxs<T: Flatten>(f: &FlatView, len: usize) -> Box<[T]> {
    (0 .. len as u32).map(|i| f.unflatten(i))
        .collect::<Vec<_>>().into_boxed_slice()
}
