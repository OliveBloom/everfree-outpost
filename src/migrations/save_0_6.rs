/// Library for reading v0.6 save files, specifically as of release-2015-12-05c.

extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;

use std::collections::HashMap;
use std::io;
use std::iter;
use std::mem;
use std::slice;

use server_extra as extra;
use server_extra::Extra;
use server_types::*;
use server_util::bytes::{Bytes, ReadBytes};
use server_world_types::{Motion, Item};
use server_world_types::flags;


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

// Objects

pub type SaveId = u32;

pub struct World {
    pub next_client: u64,
    pub next_entity: u64,
    pub next_inventory: u64,
    pub next_plane: u64,
    pub next_terrain_chunk: u64,
    pub next_structure: u64,

    pub extra: Extra,
    pub child_entities: Vec<Entity>,
    pub child_inventories: Vec<Inventory>,
}

pub struct Client {
    pub id: SaveId,
    pub stable_id: StableId,

    pub pawn_id: Option<SaveId>,

    pub extra: Extra,
    pub child_entities: Vec<Entity>,
    pub child_inventories: Vec<Inventory>,
}

pub struct Entity {
    pub id: SaveId,
    pub stable_id: StableId,

    pub stable_plane: Stable<PlaneId>,
    pub motion: Motion,
    pub anim: AnimId,
    pub facing: V3,
    pub target_velocity: V3,
    pub appearance: u32,

    pub extra: Extra,
    pub child_inventories: Vec<Inventory>,
}

pub struct Inventory {
    pub id: SaveId,
    pub stable_id: StableId,

    pub contents: Box<[Item]>,

    pub extra: Extra,
}

pub struct Plane {
    pub id: SaveId,
    pub stable_id: StableId,

    pub name: String,
    pub saved_chunks: HashMap<V2, Stable<TerrainChunkId>>,

    pub extra: Extra,
}

pub struct TerrainChunk {
    pub id: SaveId,
    pub stable_id: StableId,

    pub flags: flags::TerrainChunkFlags,
    pub blocks: Box<BlockChunk>,
    pub block_names: HashMap<BlockId, String>,

    pub child_structures: Vec<Structure>,
}

pub struct Structure {
    pub id: SaveId,
    pub stable_id: StableId,

    pub pos: V3,
    pub template_id: TemplateId,
    pub flags: flags::StructureFlags,

    pub extra: Extra,
    pub child_inventories: Vec<Inventory>,
}

// Reader

pub struct Reader<R: io::Read> {
    r: R,

    item_names: HashMap<ItemId, String>,
    template_names: HashMap<TemplateId, String>,
}

impl<R: io::Read> Reader<R> {
    pub fn new(r: R) -> Reader<R> {
        Reader {
            r: r,
            item_names: HashMap::new(),
            template_names: HashMap::new(),
        }
    }


    pub fn take_item_names(&mut self) -> HashMap<ItemId, String> {
        mem::replace(&mut self.item_names, HashMap::new())
    }

    pub fn take_template_names(&mut self) -> HashMap<TemplateId, String> {
        mem::replace(&mut self.template_names, HashMap::new())
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


    pub fn read_header(&mut self) -> io::Result<SaveHeader> {
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


    pub fn read_world(&mut self) -> io::Result<World> {
        let next_client = try!(self.r.read_bytes());
        let next_entity = try!(self.r.read_bytes());
        let next_inventory = try!(self.r.read_bytes());
        let next_plane = try!(self.r.read_bytes());
        let next_terrain_chunk = try!(self.r.read_bytes());
        let next_structure = try!(self.r.read_bytes());

        let extra = try!(self.read_extra());


        let mut child_entities = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_entities.push(try!(self.read_entity()));
        }
        let mut child_inventories = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_inventories.push(try!(self.read_inventory()));
        }

        Ok(World {
            next_client: next_client,
            next_entity: next_entity,
            next_inventory: next_inventory,
            next_plane: next_plane,
            next_terrain_chunk: next_terrain_chunk,
            next_structure: next_structure,

            extra: extra,
            child_entities: child_entities,
            child_inventories: child_inventories,
        })
    }

    pub fn read_client(&mut self) -> io::Result<Client> {
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

    pub fn read_entity(&mut self) -> io::Result<Entity> {
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

    pub fn read_inventory(&mut self) -> io::Result<Inventory> {
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

    pub fn read_plane(&mut self) -> io::Result<Plane> {
        let id = try!(self.r.read_bytes());
        let stable_id = try!(self.r.read_bytes());

        let name_len = try!(self.read_count());
        let name = try!(self.read_string(name_len));

        let mut saved_chunks = HashMap::<V2, Stable<TerrainChunkId>>::new();
        for _ in 0 .. try!(self.read_count()) {
            let k = try!(self.r.read_bytes());
            let v = try!(self.r.read_bytes());
            saved_chunks.insert(k, v);
        }

        let extra = try!(self.read_extra());

        Ok(Plane {
            id: id,
            stable_id: stable_id,

            name: name,
            saved_chunks: saved_chunks,

            extra: extra,
        })
    }

    pub fn read_terrain_chunk(&mut self) -> io::Result<TerrainChunk> {
        let id = try!(self.r.read_bytes());
        let stable_id = try!(self.r.read_bytes());

        let flags_raw = try!(self.r.read_bytes());
        let flags = unwrap!(flags::TerrainChunkFlags::from_bits(flags_raw));

        let mut blocks = Box::new(EMPTY_CHUNK);
        let slice = unsafe {
            slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8,
                                      mem::size_of::<BlockChunk>())
        };
        try!(self.r.read_exact(slice));

        let mut block_names = HashMap::new();
        for _ in 0 .. try!(self.read_count()) {
            let (id, _, name_len): (u16, u8, u8) = unsafe { try!(self.r.read_as_bytes()) };
            let name = try!(self.read_string(name_len as usize));
            block_names.insert(id, name);
        }

        let mut child_structures = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_structures.push(try!(self.read_structure()));
        }

        Ok(TerrainChunk {
            id: id,
            stable_id: stable_id,

            flags: flags,
            blocks: blocks,
            block_names: block_names,

            child_structures: child_structures,
        })
    }

    pub fn read_structure(&mut self) -> io::Result<Structure> {
        let id = try!(self.r.read_bytes());
        let stable_id = try!(self.r.read_bytes());

        let pos = try!(self.r.read_bytes());

        let template_id = try!(self.r.read_bytes());
        if !self.template_names.contains_key(&template_id) {
            let (_, _, _, name_len): (u8, u8, u8, u8) = unsafe { try!(self.r.read_as_bytes()) };
            let name = try!(self.read_string(name_len as usize));
            self.template_names.insert(template_id, name);
        }

        let flags_raw = try!(self.r.read_bytes());
        let flags = unwrap!(flags::StructureFlags::from_bits(flags_raw));

        let extra = try!(self.read_extra());

        let mut child_inventories = Vec::new();
        for _ in 0 .. try!(self.read_count()) {
            child_inventories.push(try!(self.read_inventory()));
        }

        Ok(Structure {
            id: id,
            stable_id: stable_id,

            pos: pos,
            template_id: template_id,
            flags: flags,

            extra: extra,
            child_inventories: child_inventories,
        })
    }
}



fn dump_extra(e: &Extra) {
    println!(" == extra");
    for (k, v) in e.iter() {
        print!("{:?}: ", k);
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, 1),
            extra::View::Hash(h) => dump_extra_hash(h, 1),
        }
    }
    println!(" ==");
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
    use std::env;

    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);

    let mut f = Reader::new(File::open(&args[1]).unwrap());
    let h = f.read_header().unwrap();
    println!("found version: {}.{}", h.major, h.minor);

    let tc = f.read_terrain_chunk().unwrap();
    println!("read terrain chunk: {:x}", tc.stable_id);

    println!("found {} child structures", tc.child_structures.len());
    for s in &tc.child_structures {
        println!("  {} @ {:?}", f.template_names[&s.template_id], s.pos);
    }

    /*
    let p = f.read_plane().unwrap();
    println!("read plane: {} ({:x})", p.name, p.stable_id);
    dump_extra(&p.extra);

    println!("  plane has {} saved chunks", p.saved_chunks.len());
    */

    /*
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
            println!("read inventory: {:x}", i.id);
            dump_extra(&i.extra);
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
    */
}
