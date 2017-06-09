/// Remove `cave_detail` and `cave_junk` files from the terrain gen summary, for chunks whose
/// neighbors have all been generated.  This frees up inodes in the underlying file system.  Note
/// that the summary files will be removed in-place.
///
/// Usage: ./2017-06-08-remove-cave-detail-summary save-dir

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
use server_bundle::types::Bundle;
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

fn for_each_subdir<F: FnMut(&Path)>(base: &str, mut f: F) {
    for ent in fs::read_dir(base).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_dir() {
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

fn read_plane_bundles(dir_in: &str) -> HashMap<Stable<PlaneId>, Bundle> {
    let mut h = HashMap::new();

    for_each_file(&format!("{}/planes", dir_in), |path| {
        let b = read_bundle(path);
        let stable = b.planes[0].stable_id;
        h.insert(Stable::new(stable), b);
    });

    h
}


fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let save_dir = &args[1];

    let mut plane_bundles = read_plane_bundles(save_dir);

    for (stable_pid, bundle) in plane_bundles {
        let plane = &bundle.planes[0];

        let mut seen_neighbors = HashMap::with_capacity(plane.saved_chunks.len());
        for &(cpos, _) in plane.saved_chunks.iter() {
            for offset in Region::new(scalar(-1), scalar(2)).points() {
                *seen_neighbors.entry(cpos + offset).or_insert(0) += 1;
            }
        }

        let mut total_removed = 0;
        for (cpos, count) in seen_neighbors {
            if count != 9 {
                continue;
            }

            for &which in &["cave_detail", "cave_junk"] {
                let path = format!("{}/summary/{}/{}/{},{}",
                                   save_dir, which, stable_pid.unwrap(), cpos.x, cpos.y);
                if Path::new(&path).exists() {
                    fs::remove_dir_all(&path).unwrap();
                }
                total_removed += 1;
            }
        }
        println!("removed {} chunks for plane {:?}", total_removed, stable_pid);
    }
}

