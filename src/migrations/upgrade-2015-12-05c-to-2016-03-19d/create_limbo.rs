/// Create 1.plane (STABLE_PLANE_LIMBO).

extern crate rustc_serialize;

extern crate server_bundle;
extern crate server_config;
extern crate server_types;

use std::env;
use std::fs::File;
use std::io::Read;
use rustc_serialize::json;

use server_bundle::builder::Builder;
use server_config::{storage, data};
use server_types::*;


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
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

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let data = load_new_data(dir);

    let mut builder = Builder::new(&data);
    builder.plane()
        .name("Limbo")
        .stable_id(STABLE_PLANE_LIMBO.unwrap());

    let bundle = builder.finish();
    let mut f_out = File::create(&format!("{}/save/planes/1.plane", dir)).unwrap();
    let mut flat = server_bundle::flat::Flat::new();
    flat.flatten_bundle(&bundle);
    flat.write(&mut f_out).unwrap();
}



