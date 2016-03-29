/// Library for reading v0.6 save files, specifically as of release-2015-12-05c.

extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;

use std::collections::HashMap;
use std::io;
use std::iter;
use std::mem;

use server_extra as extra;
use server_extra::Extra;
use server_types::*;
use server_util::bytes::{Bytes, ReadBytes};
use server_world_types::{Motion, Item};


// Header

#[derive(Clone, Copy)]
pub struct SaveHeader {
    pub minor: u16,
    pub major: u16,
}
unsafe impl Bytes for SaveHeader {}

// Extra

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
        Nil =           0x00,
        Bool =          0x01,
        SmallInt =      0x02,
        LargeInt =      0x03,
        Float =         0x04,
        SmallString =   0x05,
        LargeString =   0x06,
        Table =         0x07,

        World =         0x10,
        Client =        0x11,
        Entity =        0x12,
        Inventory =     0x13,
        Structure =     0x14,

        StableClient =      0x20,
        StableEntity =      0x21,
        StableInventory =   0x22,
        StablePlane =       0x23,
        StableStructure =   0x24,

        V3 =            0x30,
        TimeU =         0x31,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ExtraWord {
    tag: Tag,
    a: u8,
    b: u16,
}

// Client

pub type SaveId = u32;

pub struct Client {
    id: SaveId,
    stable_id: StableId,

    pawn_id: Option<SaveId>,

    extra: Extra,
    child_entities: Vec<Entity>,
    child_inventories: Vec<Inventory>,
}

pub struct Entity {
    id: SaveId,
    stable_id: StableId,

    stable_plane: Stable<PlaneId>,
    motion: Motion,
    anim: AnimId,
    facing: V3,
    target_velocity: V3,
    appearance: u32,

    extra: Extra,
    child_inventories: Vec<Inventory>,
}

pub struct Inventory {
    id: SaveId,
    stable_id: StableId,

    contents: Box<[Item]>,

    extra: Extra,
}

// Reader

struct Reader<R: io::Read> {
    r: R,

    item_names: HashMap<ItemId, String>,
}

impl<R: io::Read> Reader<R> {
    fn new(r: R) -> Reader<R> {
        Reader {
            r: r,
            item_names: HashMap::new(),
        }
    }


    fn read_count(&mut self) -> io::Result<usize> {
        self.r.read_bytes::<u32>().map(|x| x as usize)
    }

    fn read_string(&mut self, len: usize) -> io::Result<String> {
        let padded_len = (len + 3) & !3;
        let mut buf = iter::repeat(0).take(padded_len).collect::<Vec<_>>();
        try!(self.r.read_exact(&mut buf));
        buf.truncate(len);
        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(_) => fail!("utf8 decoding error"),
        }
    }


    fn read_header(&mut self) -> io::Result<SaveHeader> {
        self.r.read_bytes()
    }

    fn read_extra_word(&mut self) -> io::Result<ExtraWord> {
        let (tag, a, b): (u8, u8, u16) = unsafe { try!(self.r.read_as_bytes()) };
        let tag = unwrap!(Tag::from_primitive(tag));
        Ok(ExtraWord { tag: tag, a: a, b: b })
    }

    fn read_extra_string(&mut self, w: ExtraWord) -> io::Result<String> {
        let len =
            match w.tag {
                Tag::SmallString => w.b as usize,
                Tag::LargeString => try!(self.read_count()),
                _ => fail!("expected Tag::SmallString or Tag::LargeString"),
            };
        self.read_string(len)
    }

    fn read_extra_value(&mut self, w: ExtraWord) -> io::Result<extra::Value> {
        use server_extra::Value;
        let v = match w.tag {
            Tag::Nil => Value::Null,
            Tag::Bool => Value::Bool(w.a != 0),
            Tag::SmallInt => Value::Int(w.b as i16 as i64),
            Tag::LargeInt => Value::Int(try!(self.r.read_bytes())),
            Tag::Float => Value::Float(try!(self.r.read_bytes())),
            Tag::SmallString |
            Tag::LargeString => Value::Str(try!(self.read_extra_string(w))),
            Tag::Table => fail!("expected value, but saw Tag::Table"),

            Tag::World => Value::Null,
            Tag::Client => {
                let raw = try!(self.r.read_bytes::<u32>());
                Value::ClientId(ClientId(raw as u16))
            },
            Tag::Entity => Value::EntityId(try!(self.r.read_bytes())),
            Tag::Inventory => Value::InventoryId(try!(self.r.read_bytes())),
            Tag::Structure => Value::StructureId(try!(self.r.read_bytes())),

            Tag::StableClient => Value::StableClientId(try!(self.r.read_bytes())),
            Tag::StableEntity => Value::StableEntityId(try!(self.r.read_bytes())),
            Tag::StableInventory => Value::StableInventoryId(try!(self.r.read_bytes())),
            Tag::StablePlane => Value::StablePlaneId(try!(self.r.read_bytes())),
            Tag::StableStructure => Value::StableStructureId(try!(self.r.read_bytes())),

            Tag::V3 => {
                let x = try!(self.r.read_bytes());
                let y = try!(self.r.read_bytes());
                let z = try!(self.r.read_bytes());
                Value::V3(V3::new(x, y, z))
            },
            Tag::TimeU => Value::Int(try!(self.r.read_bytes())),
        };
        Ok(v)
    }

    fn read_extra_table(&mut self, mut e: extra::HashViewMut) -> io::Result<()> {
        loop {
            let kw = try!(self.read_extra_word());
            if kw.tag == Tag::Nil {
                break;
            }
            let k = try!(self.read_extra_string(kw));

            let vw = try!(self.read_extra_word());
            if vw.tag == Tag::Table {
                try!(self.read_extra_table(e.borrow().set_hash(&k)));
            } else {
                let v = try!(self.read_extra_value(vw));
                e.borrow().set(&k, v);
            }
        }
        Ok(())
    }

    fn read_extra(&mut self) -> io::Result<Extra> {
        let w = try!(self.read_extra_word());
        if w.tag == Tag::Nil {
            return Ok(Extra::new());
        } else if w.tag != Tag::Table {
            fail!("expected Tag::Nil or Tag::Table as top-level value");
        }
        // w.a/w.b are always 0 for Table.  The k/v pairs are Nil-terminated instead.

        let mut e = Extra::new();
        loop {
            let kw = try!(self.read_extra_word());
            if kw.tag == Tag::Nil {
                break;
            }
            let k = try!(self.read_extra_string(kw));

            let vw = try!(self.read_extra_word());
            if vw.tag == Tag::Table {
                try!(self.read_extra_table(e.set_hash(&k)));
            } else {
                let v = try!(self.read_extra_value(vw));
                e.set(&k, v);
            }
        }
        Ok(e)
    }


    fn read_client(&mut self) -> io::Result<Client> {
        let id = try!(self.r.read_bytes());
        let stable_id = try!(self.r.read_bytes());

        let raw_pawn_id = try!(self.r.read_bytes());
        let pawn_id = if raw_pawn_id == -1_i32 as u32 { None } else { Some(raw_pawn_id) };

        let extra = try!(self.read_extra());

        let mut child_entities = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_entities.push(try!(self.read_entity()));
        }
        let mut child_inventories = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_inventories.push(try!(self.read_inventory()));
        }

        Ok(Client {
            id: id,
            stable_id: stable_id,

            pawn_id: pawn_id,

            extra: extra,
            child_entities: child_entities,
            child_inventories: child_inventories,
        })
    }

    fn read_entity(&mut self) -> io::Result<Entity> {
        let id = try!(self.r.read_bytes());
        let stable_id = try!(self.r.read_bytes());

        let stable_plane = try!(self.r.read_bytes());
        let mut motion = Motion {
            start_pos: try!(self.r.read_bytes()),
            end_pos: try!(self.r.read_bytes()),
            start_time: try!(self.r.read_bytes()),
            duration: 0,
        };
        let (duration, anim) = unsafe { try!(self.r.read_as_bytes()) };
        motion.duration = duration;

        let facing = try!(self.r.read_bytes());
        let target_velocity = try!(self.r.read_bytes());
        let appearance = try!(self.r.read_bytes());

        let extra = try!(self.read_extra());

        let mut child_inventories = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_inventories.push(try!(self.read_inventory()));
        }

        Ok(Entity {
            id: id,
            stable_id: stable_id,

            stable_plane: stable_plane,
            motion: motion,
            anim: anim,
            facing: facing,
            target_velocity: target_velocity,
            appearance: appearance,

            extra: extra,
            child_inventories: child_inventories,
        })
    }

    fn read_inventory(&mut self) -> io::Result<Inventory> {
        let id = try!(self.r.read_bytes());
        let stable_id = try!(self.r.read_bytes());

        let size = try!(self.read_count());
        println!("reading {} items", size);
        let mut contents = server_util::make_array(Item::Empty, size);
        let mut idx = 0;
        while idx < size {
            let (tag, x, item_id): (u8, u8, ItemId) = unsafe { try!(self.r.read_as_bytes()) };
            match tag {
                0 => {
                    idx += 1;
                },
                1 => {
                    contents[idx] = Item::Bulk(x, item_id);
                    idx += 1;
                },
                255 => {
                    let len = x as usize;
                    let name = try!(self.read_string(len));
                    self.item_names.insert(item_id, name);
                    // No `idx` increment
                },
                _ => fail!("unrecognized item tag in invenotry"),
            }
        }

        let extra = try!(self.read_extra());

        Ok(Inventory {
            id: id,
            stable_id: stable_id,

            contents: contents,

            extra: extra,
        })
    }
}



fn dump_extra(e: &Extra) {
    for (k, v) in e.iter() {
        print!("{:?}: ", k);
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, 1),
            extra::View::Hash(h) => dump_extra_hash(h, 1),
        }
    }
}

fn dump_extra_value(v: extra::Value) {
    use server_extra::Value;
    match v {
        Value::Null => println!("!!null _"),
        Value::Bool(b) => println!("!!bool {}", b),
        Value::Int(i) => println!("{}", i),
        Value::Float(f) => println!("{}", f),
        Value::Str(s) => println!("{:?}", s),

        Value::ClientId(id) => println!("!!client {}", id.unwrap()),
        Value::EntityId(id) => println!("!!entity {}", id.unwrap()),
        Value::InventoryId(id) => println!("!!inventory {}", id.unwrap()),
        Value::PlaneId(id) => println!("!!plane {}", id.unwrap()),
        Value::TerrainChunkId(id) => println!("!!terrain_chunk {}", id.unwrap()),
        Value::StructureId(id) => println!("!!structure {}", id.unwrap()),

        Value::StableClientId(id) => println!("!!stable_client {}", id.unwrap()),
        Value::StableEntityId(id) => println!("!!stable_entity {}", id.unwrap()),
        Value::StableInventoryId(id) => println!("!!stable_inventory {}", id.unwrap()),
        Value::StablePlaneId(id) => println!("!!stable_plane {}", id.unwrap()),
        Value::StableTerrainChunkId(id) => println!("!!stable_terrain_chunk {}", id.unwrap()),
        Value::StableStructureId(id) => println!("!!stable_structure {}", id.unwrap()),

        Value::V2(v) => println!("!!v2 [{}, {}]", v.x, v.y),
        Value::V3(v) => println!("!!v3 [{}, {}, {}]", v.x, v.y, v.z),
        Value::Region2(r) => println!("!!region2 [[{}, {}], [{}, {}]]",
                                      r.min.x, r.min.y,
                                      r.max.x, r.max.y),
        Value::Region3(r) => println!("!!region3 [[{}, {}, {}], [{}, {}, {}]]",
                                      r.min.x, r.min.y, r.min.z,
                                      r.max.x, r.max.y, r.max.z),
    }
}

fn dump_extra_array(a: extra::ArrayView, indent: usize) {
    println!("");
    for _ in 0 .. indent {
        print!("    ");
    }
    for v in a.iter() {
        print!("- ");
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, indent + 1),
            extra::View::Hash(h) => dump_extra_hash(h, indent + 1),
        }
    }
}

fn dump_extra_hash(h: extra::HashView, indent: usize) {
    println!("");
    for _ in 0 .. indent {
        print!("    ");
    }
    for (k, v) in h.iter() {
        print!("{:?}: ", k);
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, indent + 1),
            extra::View::Hash(h) => dump_extra_hash(h, indent + 1),
        }
    }
}


fn main() {
    use std::fs::File;
    let mut f = Reader::new(File::open("Mikrokek.client").unwrap());
    let h = f.read_header().unwrap();
    println!("found version: {}.{}", h.major, h.minor);

    let c = f.read_client().unwrap();
    println!("read client: {:x}", c.stable_id);
    dump_extra(&c.extra);

    for e in &c.child_entities {
        println!("read entity: {:x}", e.id);
        println!("  facing: {:?}", e.facing);
        println!("  target_velocity: {:?}", e.target_velocity);
        println!("  appearance: {:x}", e.appearance);
        dump_extra(&e.extra);

        for i in &e.child_inventories {
            println!("read inventory: {:x}", e.id);
            for item in i.contents.iter() {
                match *item {
                    Item::Empty => continue,
                    Item::Bulk(count, item_id) =>
                        println!("  {} {}", count, f.item_names[&item_id]),
                    Item::Special(_, item_id) =>
                        println!("  1* {}", f.item_names[&item_id]),
                }
            }
        }
    }
}
