/// Convert remaining structure extras, in-place.  This fixes up the fields that are not handled by
/// convert_chunks.
///
/// Depends on the upgrade_stable_id_map.dat file produced by pin_entities.

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
use server_bundle::flat;
use server_bundle::types::Bundle;
use server_config::{storage, data};
use server_extra as extra;
use server_extra::{Extra, View, ViewMut, Value};
use server_types::*;
use server_util::bytes::{ReadBytes, WriteBytes};
use server_world_types::{Motion, Item};

use save_0_6::*;


fn load_bundle(path: &Path) -> Bundle {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let flat = flat::FlatView::from_bytes(&buf).unwrap();
    flat.unflatten_bundle()
}

fn save_bundle(path: &Path, b: &Bundle) {
    let mut f = File::create(path).unwrap();
    let mut flat = server_bundle::flat::Flat::new();
    flat.flatten_bundle(b);
    flat.write(&mut f).unwrap();
}


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

fn load_data(path: &str) -> data::Data {
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


fn copy_extra(old: View, new: &mut Extra, key: &str) {
    match old {
        View::Value(v) => new.set(key, v),
        View::Array(a) => copy_extra_array(a, new.set_array(key)),
        View::Hash(h) => copy_extra_hash(h, new.set_hash(key)),
    }
}

fn copy_extra_array(old: extra::ArrayView, mut new: extra::ArrayViewMut) {
    for (i, view) in old.iter().enumerate() {
        new.borrow().push();
        match view {
            View::Value(v) => new.borrow().set(i, v),
            View::Array(a) => copy_extra_array(a, new.borrow().set_array(i)),
            View::Hash(h) => copy_extra_hash(h, new.borrow().set_hash(i)),
        }
    }
}

fn copy_extra_hash(old: extra::HashView, mut new: extra::HashViewMut) {
    for (k, view) in old.iter() {
        match view {
            View::Value(v) => new.borrow().set(k, v),
            View::Array(a) => copy_extra_array(a, new.borrow().set_array(k)),
            View::Hash(h) => copy_extra_hash(h, new.borrow().set_hash(k)),
        }
    }
}

fn convert_extra(old: &Extra, id_map: &HashMap<u64, u64>, template: &str) -> Extra {
    let mut new = Extra::new();

    if old.get("_pending_timer_when").is_some() {
        let msg =
            if template.ends_with("/0") { Value::Int(1) }
            else if template.ends_with("/1") { Value::Int(2) }
            else if template.ends_with("/2") { Value::Int(3) }
            else if template.ends_with("/opening") { Value::Str(String::from("open")) }
            else if template.ends_with("/open") { Value::Str(String::from("closing")) }
            else if template.ends_with("/closing") { Value::Str(String::from("closed")) }
            else { panic!("don't know how to generate timer msg for {}", template) };
        let when = match old.get("_pending_timer_when") {
            Some(View::Value(Value::Int(i))) => i,
            _ => panic!("expected _pending_timer_when to be an int"),
        };

        let mut sm = new.set_hash("sm");
        {
            let mut timer = sm.borrow().set_hash("timer");
            timer.borrow().set("msg", msg);
            timer.borrow().set("when", Value::Int(when));
        }
        if template.ends_with("/0") ||
           template.ends_with("/1") ||
           template.ends_with("/2") ||
           template.ends_with("/3") {
            // Crop
            println!("  updating crop state for {}", template);
            let base = match old.get("_start_time") {
                Some(View::Value(Value::Int(i))) => i,
                _ => panic!("expected _start_time to be an int"),
            };
            let step = match old.get("_grow_time") {
                Some(View::Value(Value::Float(f))) => f as i64,
                _ => panic!("expected _grow_time to be a float"),
            };
            sm.borrow().set("base", Value::Int(base));
            sm.borrow().set("step", Value::Int(step));
        } else {
            // Door
            println!("  updating door state for {}", template);
            let state = template.split('/').next_back().unwrap();
            sm.borrow().set("state", Value::Str(String::from(state)));
            sm.borrow().set("auto", Value::Bool(true));
        }
    }

    for (k, v) in old.iter() {
        match k {
            // Ward owner.  Convert client StableId to entity StableId.
            "_owner_cid" => {
                if let View::Value(Value::StableClientId(cid)) = v {
                    let eid = Stable::new(id_map[&cid.unwrap()]);
                    println!("  updated ward owner: {:?} -> {:?}", cid, eid);
                    new.set("owner", Value::StableEntityId(eid));
                } else {
                    panic!("expected _owner_cid to be a StableClientId");
                }
            },

            "_pending_timer_when" => {},
            "_start_time" => {},
            "_grow_time" => {},

            _ => copy_extra(v, &mut new, k),
        }
    }

    new
}



fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let data = load_data(dir);

    let mut id_map = HashMap::new();
    {
        let mut f = File::open("upgrade_stable_id_map.dat").unwrap();
        for _ in 0 .. f.read_bytes::<u32>().unwrap() {
            let cid = f.read_bytes::<u64>().unwrap();
            let eid = f.read_bytes::<u64>().unwrap();
            id_map.insert(cid, eid);
        }
    }


    for ent in fs::read_dir(&format!("{}/save/terrain_chunks", dir)).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_file() {
            continue;
        }
        println!("load: {:?}", ent.path());
        let mut bundle = load_bundle(&ent.path());

        let mut changed = false;
        for s in bundle.structures.iter_mut() {
            if s.extra.len() == 0 {
                continue;
            }

            let template_id = s.template;
            let template = &*bundle.templates[template_id as usize];

            let new_extra = convert_extra(&s.extra, &id_map, template);
            s.extra = new_extra;
            changed = true;
        }

        if changed {
            save_bundle(&ent.path(), &bundle);
        }
    }
}
