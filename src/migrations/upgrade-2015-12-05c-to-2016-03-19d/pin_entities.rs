/// Assign a StableId to each Entity, and save a map from client StableIds to the corresponding
/// entity StableIds.  The update is performed in-place, on new (bundle-format) .client files.
///
/// The StableId map is used when converting ward data.  The old version uses client StableIds,
/// while the new version uses entity StableIds.

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
use server_bundle::types::Bundle;
use server_bundle::flat;
use server_config::{storage, data};
use server_extra::{Extra, View, ViewMut, Value};
use server_types::*;
use server_util::bytes::WriteBytes;
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

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let world_path = format!("{}/save/world.dat", dir);
    let mut world_bundle = load_bundle(Path::new(&world_path));

    let mut map = HashMap::new();

    {
        let world = world_bundle.world.as_mut().unwrap();
        let mut next_id = world.next_entity;

        for ent in fs::read_dir(&format!("{}/save/clients", dir)).unwrap() {
            let ent = ent.unwrap();
            if !ent.file_type().unwrap().is_file() {
                continue;
            }
            let path = ent.path();

            let mut bundle = load_bundle(&path);
            {
                assert!(bundle.clients.len() == 1);
                let c = &mut bundle.clients[0];
                if c.pawn.is_none() {
                    continue;
                }
                if c.stable_id == NO_STABLE_ID {
                    continue;
                }

                let e = &mut bundle.entities[c.pawn.unwrap().unwrap() as usize];
                e.stable_id = next_id;
                next_id += 1;
                println!("map: {} -> {}", c.stable_id, e.stable_id);
                map.insert(c.stable_id, e.stable_id);
            }
            save_bundle(&path, &bundle);
        }
    }

    save_bundle(Path::new(&world_path), &world_bundle);


    let map_vec = map.into_iter().collect::<Vec<_>>();
    let mut f = File::create("upgrade_stable_id_map.dat").unwrap();
    f.write_bytes(map_vec.len() as u32).unwrap();
    for (k, v) in map_vec {
        f.write_bytes(k).unwrap();
        f.write_bytes(v).unwrap();
    }
}



