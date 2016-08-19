/// Move all player characters to the default spawn point (32, 32, 0).
///
/// Usage: ./move-to-spawn old_clients new_clients

extern crate server_bundle;
extern crate server_types;

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use server_bundle::flat::{FlatViewMut, CV3};
use server_types::*;


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

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir_in = &args[1];
    let dir_out = &args[2];

    for_each_file(dir_in, |path| {
        println!("\nprocessing {:?}", path);
        let path_out = Path::new(dir_out).join(path.file_name().unwrap());

        let mut buf = Vec::new();
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();

        {
            let view = FlatViewMut::from_bytes(&mut buf).unwrap();

            for e in view.entities {
                e.stable_plane = STABLE_PLANE_FOREST.unwrap();
                e.motion_start_time = 0;
                e.motion_end_time = TIME_MIN;
                e.motion_start_pos = CV3 { x: 32, y: 32, z: 0 };
                e.motion_velocity = CV3 { x: 0, y: 0, z: 0 };
                e.target_velocity = CV3 { x: 0, y: 0, z: 0 };
            }
        }

        File::create(&path_out).unwrap()
            .write_all(&buf).unwrap();
        println!("wrote output to {:?}", path_out);
    });
}
