/// Convert world extras to the new format.  Note that only some of the new extras are written into
/// world.dat - others are written into 2.plane instead.  Both files must already exist in the
/// output directory.

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


fn load_world(path: &str) -> World {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_world().unwrap()
}

fn load_bundle(path: &str) -> Bundle {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let flat = flat::FlatView::from_bytes(&buf).unwrap();
    flat.unflatten_bundle()
}

fn save_bundle(path: &str, b: &Bundle) {
    let mut f = File::create(path).unwrap();
    let mut flat = server_bundle::flat::Flat::new();
    flat.flatten_bundle(b);
    flat.write(&mut f).unwrap();
}


fn convert_teleport_networks(old: View, mut new: extra::HashViewMut) {
    let old = match old {
        View::Hash(h) => h,
        _ => panic!("expected teleport_networks to be a hash"),
    };
    for (net, teles) in old.iter() {
        let teles = match teles {
            View::Hash(h) => h,
            _ => panic!("expected teleport_networks[{:?}] to be a hash", net),
        };
        let mut new_teles = new.borrow().set_hash(net);
        for (name, pos) in teles.iter() {
            let pos = match pos {
                View::Value(Value::V3(v)) => v,
                _ => panic!("expected teleport_networks[{:?}][{:?}] to be a V3", net, name),
            };
            new_teles.borrow().set(name, Value::V3(pos));
        }
    }
}

fn convert_ward_perm(old: View,
                     mut new: extra::HashViewMut,
                     id_map: &HashMap<u64, u64>) {
    let old = match old {
        View::Hash(h) => h,
        _ => panic!("expected ward_perm to be a hash"),
    };
    for (key, perms) in old.iter() {
        let perms = match perms {
            View::Hash(h) => h,
            _ => panic!("expected ward_perm[{:?}] to be a hash", key),
        };

        let old_key_val = u64::from_str_radix(key, 16).unwrap();
        let new_key_val = id_map[&old_key_val];
        let new_key = format!("{}", new_key_val);

        println!("ward perm key: {} -> {}", key, new_key);
        let mut new_perms = new.borrow().set_hash(&new_key);
        for (name, ok) in perms.iter() {
            match ok {
                View::Value(Value::Bool(true)) => {},
                _ => panic!("expected ward_perm[{:?}][{:?}] to be true", key, name),
            }
            new_perms.borrow().set(name, Value::Bool(true));
        }
    }
}

fn convert_ward_info(old: View,
                     mut new: extra::HashViewMut,
                     id_map: &HashMap<u64, u64>) {
    let old = match old {
        View::Hash(h) => h,
        _ => panic!("expected ward_info to be a hash"),
    };
    for (key, info) in old.iter() {
        let info = match info {
            View::Hash(h) => h,
            _ => panic!("expected ward_info[{:?}] to be a hash", key),
        };

        let new_key =
            if key == "server" {
                String::from("server")
            } else {
                let old_key_val = u64::from_str_radix(key, 16).unwrap();
                let new_key_val = id_map[&old_key_val];
                format!("{}", new_key_val)
            };

        println!("ward info key: {} -> {}", key, new_key);
        let mut new_info = new.borrow().set_hash(&new_key);
        let name = match info.get("name") {
            Some(View::Value(Value::Str(s))) => s,
            _ => panic!("expected ward_info[{:?}][\"name\"] to be a String", key),
        };
        let pos = match info.get("pos") {
            Some(View::Value(Value::V3(v))) => v,
            _ => panic!("expected ward_info[{:?}][\"name\"] to be a V3", key),
        };
        new_info.borrow().set("name", Value::Str(String::from(name)));
        new_info.borrow().set("pos", Value::V3(pos));
    }
}

fn build_new_extra(old: Extra, id_map: &HashMap<u64, u64>) -> (Extra, Extra) {
    let mut e_world = Extra::new();
    let mut e_plane = Extra::new();
    for (k,v) in old.iter() {
        match k {
            "teleport_networks" =>
                convert_teleport_networks(v, e_world.set_hash("teleport_networks")),
            "ward_perm" =>
                convert_ward_perm(v, e_world.set_hash("ward_perm"), id_map),
            "ward_info" =>
                convert_ward_info(v, e_plane.set_hash("ward_info"), id_map),
            _ => {
                // Only falls through if the key/value was unrecognized.
                match v {
                    View::Value(v) => println!("  unrecognized: {} => {:?}", k, v),
                    View::Array(_) => println!("  unrecognized: {} => <array>", k),
                    View::Hash(_) => println!("  unrecognized: {} => <hash>", k),
                }
            },
        }
    }
    (e_world, e_plane)
}


fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let old_dir = &args[1];
    let new_dir = &args[2];


    let mut id_map = HashMap::new();
    {
        let mut f = File::open("upgrade_stable_id_map.dat").unwrap();
        for _ in 0 .. f.read_bytes::<u32>().unwrap() {
            let cid = f.read_bytes::<u64>().unwrap();
            let eid = f.read_bytes::<u64>().unwrap();
            id_map.insert(cid, eid);
        }
    }


    let world = load_world(&format!("{}/save/world.dat", old_dir));

    let (world_extra, plane_extra) = build_new_extra(world.extra, &id_map);

    println!("writing world.dat");
    let mut bundle = load_bundle(&format!("{}/save/world.dat", new_dir));
    {
        let world = bundle.world.as_mut().unwrap();
        world.extra = world_extra;
    }
    save_bundle(&format!("{}/save/world.dat", new_dir), &bundle);

    println!("writing 2.plane");
    let mut bundle = load_bundle(&format!("{}/save/planes/2.plane", new_dir));
    {
        let plane = &mut bundle.planes[0];
        //assert!(plane.extra.len() == 0);
        plane.extra = plane_extra;
    }
    save_bundle(&format!("{}/save/planes/2.plane", new_dir), &bundle);
}



