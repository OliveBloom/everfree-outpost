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
use std::path::Path;
use rustc_serialize::json;

use physics::{CHUNK_SIZE, TILE_SIZE};
use server_bundle::builder::Builder;
use server_config::{storage, data};
use server_extra::{Extra, View, Value};
use server_types::*;
use server_util::bytes::WriteBytes;
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



fn load_client(path: &Path) -> Client {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_client().unwrap()
}

enum AnyId {
    Client(ClientId),
    Entity(EntityId),
    Inventory(InventoryId),
}

impl AnyId {
    fn as_client(&self) -> ClientId {
        match *self {
            AnyId::Client(id) => id,
            _ => panic!("expected ClientId"),
        }
    }

    fn as_entity(&self) -> EntityId {
        match *self {
            AnyId::Entity(id) => id,
            _ => panic!("expected EntityId"),
        }
    }

    fn as_inventory(&self) -> InventoryId {
        match *self {
            AnyId::Inventory(id) => id,
            _ => panic!("expected InventoryId"),
        }
    }
}


fn build_anim_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for anim in &["stand", "walk", "run"] {
        for dir in 0 .. 8 {
            map.insert(format!("pony/{}-{}", anim, dir),
                       format!("pony//{}-{}", anim, dir));
        }
    }
    map
}


struct Context<'d: 'b, 'b> {
    data: &'d data::Data,
    id_map: HashMap<SaveId, AnyId>,
    builder: &'b mut Builder<'d>,

    anim_map: HashMap<String, String>,
}

impl<'d, 'b> Context<'d, 'b> {
    fn new(data: &'d data::Data, builder: &'b mut Builder<'d>) -> Context<'d, 'b> {
        Context {
            data: data,
            id_map: HashMap::new(),
            builder: builder,
            anim_map: build_anim_map(),
        }
    }


    fn pre_client(&mut self, c: &Client) {
        let cid = self.builder.client().id();
        self.id_map.insert(c.id, AnyId::Client(cid));

        for e in &c.child_entities {
            self.pre_entity(e);
        }

        for i in &c.child_inventories {
            self.pre_inventory(i);
        }
    }

    fn pre_entity(&mut self, e: &Entity) {
        let eid = self.builder.entity().id();
        self.id_map.insert(e.id, AnyId::Entity(eid));

        for i in &e.child_inventories {
            self.pre_inventory(i);
        }
    }

    fn pre_inventory(&mut self, i: &Inventory) {
        let iid = self.builder.inventory().id();
        self.id_map.insert(i.id, AnyId::Inventory(iid));
    }


    fn convert_client(&mut self, c: &Client) {
        {
            let cid = self.id_map[&c.id].as_client();
            let mut b = self.builder.get_client(cid);

            b.stable_id(c.stable_id);
            if let Some(save_eid) = c.pawn_id {
                b.pawn_id(self.id_map[&save_eid].as_entity());
            }
            {
                let id_map = &self.id_map;
                b.extra(|extra| convert_client_extra(&c.extra, extra, id_map));
            }
        }

        for e in &c.child_entities {
            self.convert_entity(e);
        }
        for i in &c.child_inventories {
            self.convert_inventory(i);
        }
    }

    fn convert_entity(&mut self, e: &Entity) {
        {
            let eid = self.id_map[&e.id].as_entity();
            let mut b = self.builder.get_entity(eid);

            b.stable_id(e.stable_id);
            if e.stable_plane == STABLE_PLANE_FOREST {
                b.stable_plane(e.stable_plane);
                b.motion(e.motion.clone());
            } else {
                b.stable_plane(STABLE_PLANE_FOREST);
                b.motion(Motion::stationary(scalar(32), e.motion.start_time));
            }
            let old_anim = &self.data.animations.animation(e.anim).name;
            println!("mapping old anim {}", old_anim);
            let new_anim = &self.anim_map[old_anim];
            b.anim(new_anim);
            b.facing(e.facing);
            b.appearance(e.appearance);
            {
                let id_map = &self.id_map;
                b.extra(|extra| convert_entity_extra(&e.extra, extra, id_map));
            }
        }

        for i in &e.child_inventories {
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
                        let name = &self.data.item_data.name(item_id);
                        b.item(idx as u8, name, count);
                    },
                }
            }
            assert!(i.extra.len() == 0, "inventory extras are unsupported");
        }
    }
}

fn convert_client_extra(old: &Extra, new: &mut Extra, map: &HashMap<SaveId, AnyId>) {
}

fn convert_entity_extra(old: &Extra, new: &mut Extra, map: &HashMap<SaveId, AnyId>) {
}

fn decode_name(name: &str) -> String {
    let mut s = String::new();

    let mut iter = name.chars();
    while let Some(c) = iter.next() {
        if c != '-' {
            s.push(c);
            continue;
        }

        // Otherwise, need to decode an escape.
        assert!(iter.next() == Some('x'));
        assert!(iter.next() == Some('2'));
        let d = iter.next().unwrap();
        match d {
            '0' => s.push(' '),
            'd' => s.push('-'),
            _ => panic!("unrecognized escape: {}", d),
        }
    }
    s
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let old_dir = &args[1];
    let new_dir = &args[2];

    let old_data = load_old_data(old_dir);
    let new_data = load_new_data(new_dir);

    for ent in fs::read_dir(&format!("{}/save/clients", old_dir)).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_file() {
            continue;
        }
        let path = ent.path();
        println!("loading {:?}", path);
        let c = load_client(&path);

        let mut b = Builder::new(&new_data);
        {
            let mut ctx = Context::new(&old_data, &mut b);
            ctx.pre_client(&c);
            ctx.convert_client(&c);
        }
        let enc_name = path.file_stem().unwrap().to_str().unwrap();
        let name = decode_name(enc_name);
        b.get_client(ClientId(0)).name(&name);

        let bundle = b.finish();
        let path_out = format!("{}/save/clients/{}.client", new_dir, enc_name);
        let mut f_out = File::create(&path_out).unwrap();
        let mut flat = server_bundle::flat::Flat::new();
        flat.flatten_bundle(&bundle);
        flat.write(&mut f_out).unwrap();
    }
}



