/// Convert all clients from the old format to the new one.

extern crate rustc_serialize;

extern crate physics;
extern crate server_bundle;
extern crate server_config;
extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;

extern crate save_0_6;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::iter;
use std::path::Path;
use rustc_serialize::json;

use physics::{CHUNK_SIZE, TILE_SIZE};
use server_bundle::builder::Builder;
use server_config::{storage, data};
use server_extra::{Extra, View, Value};
use server_types::*;
use server_util::bytes::{ReadBytes, WriteBytes};
use server_world_types::{Motion, Item};

use save_0_6::*;


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

fn load_old_data(path: &str) -> data::Data {
    let storage = storage::Storage::new(&path);

    let block_json = read_json(storage.open_block_data());
    let item_json = read_json(storage.open_item_data());
    let recipe_json = read_json(storage.open_recipe_data());
    let template_json = read_json(storage.open_template_data());
    let animation_json = read_json(storage.open_animation_data());
    let sprite_part_json = json::Json::Array(Vec::new());
    let loot_table_json = read_json(storage.open_loot_table_data());
    let data = data::Data::from_json(block_json,
                                     item_json,
                                     recipe_json,
                                     template_json,
                                     animation_json,
                                     sprite_part_json,
                                     loot_table_json).unwrap();

    data
}

fn load_new_data(path: &str) -> data::Data {
    let storage = storage::Storage::new(&path);

    let block_json = read_json(storage.open_block_data());
    let item_json = read_json(storage.open_item_data());
    let recipe_json = read_json(storage.open_recipe_data());
    let template_json = read_json(storage.open_template_data());
    let animation_json = read_json(storage.open_animation_data());
    let sprite_part_json = read_json(storage.open_sprite_part_data());
    let loot_table_json = read_json(storage.open_loot_table_data());
    let data = data::Data::from_json(block_json,
                                     item_json,
                                     recipe_json,
                                     template_json,
                                     animation_json,
                                     sprite_part_json,
                                     loot_table_json).unwrap();

    data
}



fn load_plane(path: &str) -> Plane {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_plane().unwrap()
}

fn load_chunk(path: &str) -> TerrainChunk {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_terrain_chunk().unwrap()
}

enum AnyId {
    TerrainChunk(TerrainChunkId),
    Structure(StructureId),
    Inventory(InventoryId),
}

impl AnyId {
    fn as_terrain_chunk(&self) -> TerrainChunkId {
        match *self {
            AnyId::TerrainChunk(id) => id,
            _ => panic!("expected TerrainChunkId"),
        }
    }

    fn as_structure(&self) -> StructureId {
        match *self {
            AnyId::Structure(id) => id,
            _ => panic!("expected StructureId"),
        }
    }

    fn as_inventory(&self) -> InventoryId {
        match *self {
            AnyId::Inventory(id) => id,
            _ => panic!("expected InventoryId"),
        }
    }
}


fn build_item_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    {
        let mut add = |old, new| map.insert(String::from(old), String::from(new));
        add("fence_tee", "fence");
    }
    map
}

fn build_block_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    {
        //let mut add = |old, new| map.insert(String::from(old), String::from(new));
        for i in 0 .. 81 {
            for &base in &["grass", "dirt"] {
                // Use terrain/[gc]*/c* for both cave/*/z0/grass and .../dirt, since cave tops are
                // now covered in grass.
                let old = format!("cave/{}/z0/{}", i, base);
                let new = format!("terrain/{}/c{}",
                                  repack4_3(i, |x, s| s.push(if x == 1 { 'g' } else { 'c' })),
                                  repack4_3(i, |x, s| s.push_str(&format!("{}", x))));
                map.insert(old, new);
            }

            let old = format!("cave/{}/z1", i);
            let new = format!("cave_z1/c{}",
                              repack4_3(i, |x, s| s.push_str(&format!("{}", x))));
            map.insert(old, new);


            for &side in &["left", "right", "center"] {
                for &base in &["grass", "dirt"] {
                    let old = format!("cave/entrance/{}/{}/z0/{}", side, i, base);
                    // TODO
                    map.insert(old, String::from("empty"));
                }

                let old = format!("cave/entrance/{}/{}/z1", side, i);
                // TODO
                map.insert(old, String::from("empty"));
            }

            let old = format!("natural_ramp/back/{}", i);
            // TODO
            map.insert(old, String::from("empty"));

            for &side in &["left", "right"] {
                let old = format!("natural_ramp/{}/{}/z1", side, i);
                // TODO
                map.insert(old, String::from("empty"));
            }
        }

        for i in 0 .. 16 {
            let old = format!("cave_top/{}", i);
            let new = format!("terrain/gggg/e{}",
                              repack4_2(i, |x, s| s.push(if x == 1 { '0' } else { '1' })));
            map.insert(old, new);
        }

        for i in 0 .. 4 {
            let old = format!("grass/center/v{}", i);
            let new = format!("terrain/gggg/v{}", i);
            map.insert(old, new);
        }


        for &level in &["z0/grass", "z0/dirt", "z1"] {
            let old = format!("natural_ramp/ramp/{}", level);
            // TODO
            map.insert(old, String::from("empty"));
        }

        let old = format!("natural_ramp/top");
        // TODO
        map.insert(old, String::from("empty"));
    }
    map
}

fn repack4_2<F: Fn(i32, &mut String)>(i: i32, f: F) -> String {
    let mut s = String::new();
    f(i % 2, &mut s);
    f(i / 2 % 2, &mut s);
    f(i / (2 * 2) % 2, &mut s);
    f(i / (2 * 2 * 2) % 2, &mut s);
    s
}

fn repack4_3<F: Fn(i32, &mut String)>(i: i32, f: F) -> String {
    let mut s = String::new();
    f(i % 3, &mut s);
    f(i / 3 % 3, &mut s);
    f(i / (3 * 3) % 3, &mut s);
    f(i / (3 * 3 * 3) % 3, &mut s);
    s
}


struct Context<'d: 'b, 'b> {
    data: &'d data::Data,
    id_map: HashMap<SaveId, AnyId>,
    builder: &'b mut Builder<'d>,

    item_map: HashMap<String, String>,
    block_map: HashMap<String, String>,
}

impl<'d, 'b> Context<'d, 'b> {
    fn new(data: &'d data::Data, builder: &'b mut Builder<'d>) -> Context<'d, 'b> {
        Context {
            data: data,
            id_map: HashMap::new(),
            builder: builder,

            item_map: build_item_map(),
            block_map: build_block_map(),
        }
    }


    fn pre_chunk(&mut self, tc: &TerrainChunk) -> TerrainChunkId {
        let tcid = self.builder.terrain_chunk().id();
        self.id_map.insert(tc.id, AnyId::TerrainChunk(tcid));

        for s in &tc.child_structures {
            let sid = self.pre_structure(s);
            self.builder.get_terrain_chunk(tcid).child_structure(sid);
        }

        tcid
    }

    fn pre_structure(&mut self, s: &Structure) -> StructureId {
        let sid = self.builder.structure().id();
        self.id_map.insert(s.id, AnyId::Structure(sid));

        for i in &s.child_inventories {
            let iid = self.pre_inventory(i);
            self.builder.get_structure(sid).child_inventory(iid);
        }

        sid
    }

    fn pre_inventory(&mut self, i: &Inventory) -> InventoryId {
        let iid = self.builder.inventory().id();
        self.id_map.insert(i.id, AnyId::Inventory(iid));

        iid
    }


    fn convert_chunk(&mut self, tc: &TerrainChunk, cpos: V2) {
        {
            let tcid = self.id_map[&tc.id].as_terrain_chunk();
            let mut b = self.builder.get_terrain_chunk(tcid);

            b.stable_id(tc.stable_id);
            b.stable_plane(STABLE_PLANE_FOREST);
            b.cpos(cpos);
            b.flags(tc.flags);

            let bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
            for pos in bounds.points() {
                let block = tc.blocks[bounds.index(pos)];
                let name = self.data.block_data.name(block);
                let new_name = self.block_map.get(name).map(|x| x as &str).unwrap_or(name);
                b.block(pos, new_name);
            }
        }

        for s in &tc.child_structures {
            self.convert_structure(s, cpos);
        }
    }

    fn convert_structure(&mut self, s: &Structure, cpos: V2) {
        {
            let sid = self.id_map[&s.id].as_structure();
            let mut b = self.builder.get_structure(sid);

            b.stable_id(s.stable_id);
            b.stable_plane(STABLE_PLANE_FOREST);
            b.pos(s.pos + cpos.extend(0) * scalar(CHUNK_SIZE));

            let name = &self.data.structure_templates.template(s.template_id).name;
            b.template(name);

            b.flags(s.flags);

            {
                let id_map = &self.id_map;
                b.extra(|extra| convert_structure_extra(&s.extra, extra, id_map));
            }
        }

        for i in &s.child_inventories {
            self.convert_inventory(i);
        }
    }

    fn convert_inventory(&mut self, i: &Inventory) {
        {
            let iid = self.id_map[&i.id].as_inventory();
            let mut b = self.builder.get_inventory(iid);

            b.stable_id(i.stable_id);
            b.size(i.contents.len() as u8);
            for (idx, item) in i.contents.iter().enumerate() {
                match *item {
                    Item::Empty => {},
                    Item::Special(_, _) => panic!("Item::Special is unsupported"),
                    Item::Bulk(count, item_id) => {
                        let name = self.data.item_data.name(item_id);
                        let new_name = self.item_map.get(name).map(|x| x as &str).unwrap_or(name);
                        b.item(idx as u8, new_name, count);
                    },
                }
            }
            assert!(i.extra.len() == 0, "inventory extras are unsupported");
        }
    }
}

fn convert_structure_extra(old: &Extra, new: &mut Extra, map: &HashMap<SaveId, AnyId>) {
    for (k,v) in old.iter() {
        match k {
            "inventory_contents" => {
                if let View::Value(Value::InventoryId(iid)) = v {
                    let new_iid = map[&iid.unwrap()].as_inventory();
                    new.set_hash("inv").set("main", Value::InventoryId(new_iid));
                    continue;
                }
            },
            "message" => {
                if let View::Value(Value::Str(s)) = v {
                    new.set("message", Value::Str(s));
                    continue;
                }
            },
            "name" => {
                if let View::Value(Value::Str(s)) = v {
                    new.set("name", Value::Str(s));
                    continue;
                }
            },
            "network" => {
                if let View::Value(Value::Str(s)) = v {
                    new.set("network", Value::Str(s));
                    continue;
                }
            },
            "target_plane" => {
                if let View::Value(Value::StablePlaneId(_)) = v {
                    // Discard, so it generates a fresh dungeon the next time someone enters.
                    continue;
                }
            },

            // The rest are just copied for later processing
            "owner" => {
                if let View::Value(Value::StableClientId(cid)) = v {
                    new.set("_owner_cid", Value::StableClientId(cid));
                    continue;
                }
            },
            "pending_timer" => {
                if let View::Hash(h) = v {
                    if let Some(View::Value(Value::Int(i))) = h.get("when") {
                        new.set("_pending_timer_when", Value::Int(i));
                        continue;
                    }
                }
            },
            "start_time" => {
                if let View::Value(Value::Int(i)) = v {
                    new.set("_start_time", Value::Int(i));
                    continue;
                }
            },
            "grow_time" => {
                if let View::Value(Value::Float(f)) = v {
                    new.set("_grow_time", Value::Float(f));
                    continue;
                }
            },

            _ => {},
        }
        // Only falls through if the key/value was unrecognized.
        match v {
            View::Value(v) => println!("  unrecognized: {} => {:?}", k, v),
            View::Array(_) => println!("  unrecognized: {} => <array>", k),
            View::Hash(_) => println!("  unrecognized: {} => <hash>", k),
        }
    }
}

fn convert_plane_extra(old: &Extra, new: &mut Extra) {
    for (k,v) in old.iter() {
        // Only falls through if the key/value was unrecognized.
        match v {
            View::Value(v) => println!("  unrecognized: {} => {:?}", k, v),
            View::Array(_) => println!("  unrecognized: {} => <array>", k),
            View::Hash(_) => println!("  unrecognized: {} => <hash>", k),
        }
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let old_dir = &args[1];
    let new_dir = &args[2];

    let old_data = load_old_data(old_dir);
    let new_data = load_new_data(new_dir);

    let mut f_pres = File::open("upgrade_preserved.dat").unwrap();
    let len = f_pres.read_bytes::<u32>().unwrap() as usize;
    let mut buf = iter::repeat(scalar(0)).take(len).collect::<Vec<_>>();
    f_pres.read_bytes_slice(&mut buf).unwrap();

    let plane = load_plane(&format!("{}/save/planes/2.plane", old_dir));
    let mut plane_builder = Builder::new(&new_data);

    {
        let mut plane_b = plane_builder.plane();
        plane_b.name(&plane.name);
        plane_b.stable_id(plane.stable_id);
        plane_b.extra(|e| convert_plane_extra(&plane.extra, e));

        for &cpos in &buf {
            let stable_id = plane.saved_chunks[&cpos];
            plane_b.saved_chunk(cpos, stable_id);

            let path = format!("{}/save/terrain_chunks/{:x}.terrain_chunk",
                               old_dir, stable_id.unwrap());
            println!("loading {:?}", path);
            let tc = load_chunk(&path);

            // Convert the terrain chunk
            let mut b = Builder::new(&new_data);
            {
                let mut ctx = Context::new(&old_data, &mut b);
                ctx.pre_chunk(&tc);
                ctx.convert_chunk(&tc, cpos);
            }
            let bundle = b.finish();
            let stable_id = bundle.terrain_chunks[0].stable_id;
            let path_out = format!("{}/save/terrain_chunks/{:x}.terrain_chunk", new_dir, stable_id);
            let mut f_out = File::create(&path_out).unwrap();
            let mut flat = server_bundle::flat::Flat::new();
            flat.flatten_bundle(&bundle);
            flat.write(&mut f_out).unwrap();
        }
    }

    let bundle = plane_builder.finish();
    let mut f_out = File::create(&format!("{}/save/planes/2.plane", new_dir)).unwrap();
    let mut flat = server_bundle::flat::Flat::new();
    flat.flatten_bundle(&bundle);
    flat.write(&mut f_out).unwrap();
}



