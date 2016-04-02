/// Convert the old world.dat and misc.dat into the new world.dat format.
///
/// Note this does not convert world extras, since those actually get attached to 2.plane in the
/// new format.

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
use server_extra::{Extra, View, ViewMut, Value};
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



fn load_world(path: &str) -> World {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_world().unwrap()
}

fn load_now(path: &str) -> i64 {
    let mut f = File::open(path).unwrap();
    f.read_bytes().unwrap()
}


fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let old_dir = &args[1];
    let new_dir = &args[2];

    let old_data = load_old_data(old_dir);
    let new_data = load_new_data(new_dir);

    let world = load_world(&format!("{}/save/world.dat", old_dir));
    let now = load_now(&format!("{}/save/misc.dat", old_dir));

    let mut builder = Builder::new(&new_data);
    builder.world()
        .now(now)
        .next_client(world.next_client)
        .next_entity(world.next_entity)
        .next_inventory(world.next_inventory)
        .next_plane(world.next_plane)
        .next_terrain_chunk(world.next_terrain_chunk)
        .next_structure(world.next_structure);
    assert!(world.child_entities.len() == 0);
    assert!(world.child_inventories.len() == 0);

    let bundle = builder.finish();
    let mut f_out = File::create(&format!("{}/save/world.dat", new_dir)).unwrap();
    let mut flat = server_bundle::flat::Flat::new();
    flat.flatten_bundle(&bundle);
    flat.write(&mut f_out).unwrap();
}



