use std::collections::HashMap;

use types::*;
use util::Convert;

use world::extra::Extra;
use world::fragment::Fragment;

use super::Result;
use super::writer::Writer;
use super::reader::Reader;

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


// Everything written to the save file is 4-byte aligned, so we include up to two extra fields
// along with the tag.  The format of the tag section is: (Tag [u8], u8, u16).  Since the data is
// written little-endian, it's possible to write only Tag or only (Tag, u8).

pub trait ExtraWriter: Writer {
    fn write_extra(&mut self, e: &Extra) -> Result<()> {
        unimplemented!()
        /*
        match *e {
            Extra::Null =>
                self.write(Tag::Null as u8),
            Extra::Bool(b) =>
                self.write((Tag::Bool as u8, b as u8)),
            Extra::Int(i) =>
                match i.to_i16() {
                    Some(small_i) =>
                        self.write((Tag::SmallInt as u8, 0u8, small_i)),
                    None => {
                        try!(self.write(Tag::LargeInt as u8));
                        self.write(i)
                    },
                },
            Extra::Float(f) => {
                try!(self.write(Tag::Float as u8));
                self.write(f)
            },
            Extra::Str(ref s) => {
                try!(write_tag_and_len(self, s.len(), Tag::SmallStr, Tag::LargeStr));
                self.write_str_bytes(s)
            },

            Extra::Array(ref v) => {
                try!(write_tag_and_len(self, v.len(), Tag::SmallArray, Tag::LargeArray));
                for e in v {
                    try!(self.write_extra(e));
                }
                Ok(())
            },
            Extra::Hash(ref h) => {
                try!(write_tag_and_len(self, h.len(), Tag::SmallHash, Tag::LargeHash));
                for (k, v) in h {
                    try!(self.write_str(k));
                    try!(self.write_extra(v));
                }
                Ok(())
            },

            Extra::ClientId(id) => {
                try!(self.write(Tag::ClientId as u8));
                self.write_id(id)
            },
            Extra::EntityId(id) => {
                try!(self.write(Tag::EntityId as u8));
                self.write_id(id)
            },
            Extra::InventoryId(id) => {
                try!(self.write(Tag::InventoryId as u8));
                self.write_id(id)
            },
            Extra::PlaneId(id) => {
                try!(self.write(Tag::PlaneId as u8));
                self.write_id(id)
            },
            Extra::TerrainChunkId(id) => {
                try!(self.write(Tag::TerrainChunkId as u8));
                self.write_id(id)
            },
            Extra::StructureId(id) => {
                try!(self.write(Tag::StructureId as u8));
                self.write_id(id)
            },

            Extra::StableClientId(id) => {
                try!(self.write(Tag::StableClientId as u8));
                self.write(id.unwrap())
            },
            Extra::StableEntityId(id) => {
                try!(self.write(Tag::StableEntityId as u8));
                self.write(id.unwrap())
            },
            Extra::StableInventoryId(id) => {
                try!(self.write(Tag::StableInventoryId as u8));
                self.write(id.unwrap())
            },
            Extra::StablePlaneId(id) => {
                try!(self.write(Tag::StablePlaneId as u8));
                self.write(id.unwrap())
            },
            Extra::StableTerrainChunkId(id) => {
                try!(self.write(Tag::StableTerrainChunkId as u8));
                self.write(id.unwrap())
            },
            Extra::StableStructureId(id) => {
                try!(self.write(Tag::StableStructureId as u8));
                self.write(id.unwrap())
            },

            Extra::V2(v2) => {
                try!(self.write(Tag::V2 as u8));
                self.write(v2)
            },
            Extra::V3(v3) => {
                try!(self.write(Tag::V3 as u8));
                self.write(v3)
            },
            Extra::Region2(region2) => {
                try!(self.write(Tag::Region2 as u8));
                try!(self.write(region2.min));
                try!(self.write(region2.max));
                Ok(())
            },
            Extra::Region3(region3) => {
                try!(self.write(Tag::Region3 as u8));
                try!(self.write(region3.min));
                try!(self.write(region3.max));
                Ok(())
            },
        }
        */
    }
}

impl<W: Writer> ExtraWriter for W {}

/*
fn write_tag_and_len<W: ExtraWriter + ?Sized>(w: &mut W,
                                              len: usize,
                                              small_tag: Tag,
                                              large_tag: Tag) -> Result<()> {
    match len.to_u16() {
        Some(small_len) => {
            try!(w.write((small_tag as u8, 0u8, small_len)));
        },
        None => {
            try!(w.write(large_tag as u8));
            try!(w.write_count(len));
        },
    }
    Ok(())
}
*/


pub trait ExtraReader: Reader {
    fn read_extra<'d, F: Fragment<'d>>(&mut self, f: &mut F) -> Result<Extra> {
        let (raw_tag, a, b): (u8, u8, u16) = try!(self.read());
        let tag = unwrap!(Tag::from_primitive(raw_tag));
        match tag {
            Tag::Null =>
                Ok(Extra::Null),
            Tag::Bool =>
                Ok(Extra::Bool(a != 0)),
            Tag::SmallInt =>
                Ok(Extra::Int(b as i16 as i64)),
            Tag::LargeInt => {
                let i = try!(self.read());
                Ok(Extra::Int(i))
            },
            Tag::Float => {
                let i = try!(self.read());
                Ok(Extra::Int(i))
            },
            Tag::SmallStr =>
                read_str(self, Some(b as usize)),
            Tag::LargeStr =>
                read_str(self, None),

            Tag::SmallArray =>
                read_array(self, f, Some(b as usize)),
            Tag::LargeArray =>
                read_array(self, f, None),
            Tag::SmallHash =>
                read_hash(self, f, Some(b as usize)),
            Tag::LargeHash =>
                read_hash(self, f, None),

            Tag::ClientId => {
                let id = try!(self.read_id(f));
                Ok(Extra::ClientId(id))
            },
            Tag::EntityId => {
                let id = try!(self.read_id(f));
                Ok(Extra::EntityId(id))
            },
            Tag::InventoryId => {
                let id = try!(self.read_id(f));
                Ok(Extra::InventoryId(id))
            },
            Tag::PlaneId => {
                let id = try!(self.read_id(f));
                Ok(Extra::PlaneId(id))
            },
            Tag::TerrainChunkId => {
                let id = try!(self.read_id(f));
                Ok(Extra::TerrainChunkId(id))
            },
            Tag::StructureId => {
                let id = try!(self.read_id(f));
                Ok(Extra::StructureId(id))
            },

            Tag::StableClientId => {
                let id = try!(self.read());
                Ok(Extra::StableClientId(Stable::new(id)))
            },
            Tag::StableEntityId => {
                let id = try!(self.read());
                Ok(Extra::StableEntityId(Stable::new(id)))
            },
            Tag::StableInventoryId => {
                let id = try!(self.read());
                Ok(Extra::StableInventoryId(Stable::new(id)))
            },
            Tag::StablePlaneId => {
                let id = try!(self.read());
                Ok(Extra::StablePlaneId(Stable::new(id)))
            },
            Tag::StableTerrainChunkId => {
                let id = try!(self.read());
                Ok(Extra::StableTerrainChunkId(Stable::new(id)))
            },
            Tag::StableStructureId => {
                let id = try!(self.read());
                Ok(Extra::StableStructureId(Stable::new(id)))
            },

            Tag::V2 => {
                let v2 = try!(self.read());
                Ok(Extra::V2(v2))
            },
            Tag::V3 => {
                let v3 = try!(self.read());
                Ok(Extra::V3(v3))
            },
            Tag::Region2 => {
                let min = try!(self.read());
                let max = try!(self.read());
                Ok(Extra::Region2(Region::new(min, max)))
            },
            Tag::Region3 => {
                let min = try!(self.read());
                let max = try!(self.read());
                Ok(Extra::Region3(Region::new(min, max)))
            },
        }
    }
}

impl<R: Reader> ExtraReader for R {}

/*
fn read_str<R: ?Sized>(r: &mut R,
                       opt_len: Option<usize>) -> Result<Extra>
        where R: ExtraReader {
    let len = match opt_len {
        Some(x) => x,
        None => try!(r.read_count()),
    };
    let s = try!(r.read_str_bytes(len));
    Ok(Extra::Str(s))
}

fn read_array<'d, R: ?Sized, F>(r: &mut R,
                                f: &mut F,
                                opt_len: Option<usize>) -> Result<Extra>
        where R: ExtraReader,
              F: Fragment<'d> {
    let len = match opt_len {
        Some(x) => x,
        None => try!(r.read_count()),
    };
    let mut v = Vec::with_capacity(len);
    for _ in 0 .. len {
        let x = try!(r.read_extra(f));
        v.push(x);
    }
    Ok(Extra::Array(v))
}

fn read_hash<'d, R: ?Sized, F>(r: &mut R,
                               f: &mut F,
                               opt_len: Option<usize>) -> Result<Extra>
        where R: ExtraReader,
              F: Fragment<'d> {
    let len = match opt_len {
        Some(x) => x,
        None => try!(r.read_count()),
    };
    let mut h = HashMap::with_capacity(len);
    for _ in 0 .. len {
        let k = try!(r.read_str());
        let v = try!(r.read_extra(f));
        h.insert(k, v);
    }
    Ok(Extra::Hash(h))
}
*/
