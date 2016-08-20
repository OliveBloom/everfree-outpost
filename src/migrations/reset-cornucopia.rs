/// Move all player characters to the default spawn point (32, 32, 0).
///
/// Usage: ./move-to-spawn old_clients new_clients

extern crate server_bundle;

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use server_bundle::flat::{Flat, FlatView};


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

        let view = FlatView::from_bytes(&buf).unwrap();
        let mut b = view.unflatten_bundle();

        for e in b.entities.iter_mut() {
            e.extra.remove("used_cornucopia");
        }

        let mut flat = Flat::new();
        flat.flatten_bundle(&b);
        flat.write(&mut File::create(&path_out).unwrap()).unwrap();
        println!("wrote output to {:?}", path_out);
    });
}
