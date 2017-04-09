/// Check client bundles for colliding entity StableIds, and renumber entities if necessary.
///
/// This updates the save file in-place.
///
/// Usage: ./2017-04-09-uncollide-entity-stable-ids save-dir

extern crate server_bundle;
extern crate server_config;
extern crate server_extra;
extern crate server_types;

use server_types::*;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use server_bundle::flat::{Flat, FlatView};
use server_bundle::types::{Bundle, World};
use server_extra::Value;


fn for_each_file<F: FnMut(&Path)>(base: &str, mut f: F) {
    for ent in fs::read_dir(base).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_file() {
            continue;
        }
        let path = ent.path();
        f(&path);
    }
}

fn read_bundle<P: AsRef<Path>+?Sized>(path: &P) -> Bundle {
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();

    let view = FlatView::from_bytes(&buf).unwrap();
    view.unflatten_bundle()
}

fn write_bundle<P: AsRef<Path>+?Sized>(path: &P, b: &Bundle) {
    let mut f = Flat::new();
    f.flatten_bundle(b);
    f.write(&mut File::create(path).unwrap()).unwrap();
}

fn next_stable_entity_id(w: &mut World) -> StableId {
    let next = w.next_entity;
    w.next_entity += 1;
    next
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let save_dir = &args[1];


    let mut world_bundle = read_bundle(&format!("{}/world.dat", save_dir));

    let mut rev_map = HashMap::new();

    for_each_file(&format!("{}/clients", save_dir), |path| {
        let b = read_bundle(path);

        for e in b.entities.iter() {
            if e.stable_id != 0 {
                rev_map.entry(e.stable_id).or_insert_with(Vec::new)
                       .push(path.to_owned());
            }
        }
    });

    for (id, paths) in rev_map {
        if paths.len() <= 1 {
            continue;
        }

        println!("\nCOLLISION on {:x}", id);
        for path in paths {
            let new_id = next_stable_entity_id(world_bundle.world.as_mut().unwrap());
            println!("  reassign {:?} = {:x}", path, new_id);

            let mut b = read_bundle(&path);
            for e in b.entities.iter_mut() {
                if e.stable_id == id {
                    e.stable_id = new_id;
                }
            }
            write_bundle(&path, &b);
        }
    }

    write_bundle(&format!("{}/world.dat", save_dir), &world_bundle);
}
