/// Debugging tool.  Reads a single bundle file and prints its contents.
///
/// Usage: ./dump-bundle input-file

extern crate server_bundle;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use server_bundle::flat::FlatView;
use server_bundle::types::Bundle;

fn read_bundle<P: AsRef<Path>+?Sized>(path: &P) -> Bundle {
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();

    let view = FlatView::from_bytes(&buf).unwrap();
    view.unflatten_bundle()
}

fn print_data_map(kind: &str, names: &[Box<str>]) {
    println!("\n{} ID mapping:", kind);
    for (i, name) in names.iter().enumerate() {
        println!("  {} => {}", i, name);
    }
}

fn print_objects<T: fmt::Debug>(kind: &str, objs: &[T]) {
    if objs.len() == 0 {
        return;
    }

    println!("\n\n == {} == ", kind);

    for (i, obj) in objs.iter().enumerate() {
        println!("\n{}: {:#?}", i, obj);
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let file_in = &args[1];

    let bundle = read_bundle(file_in);

    print_data_map("Anim", &bundle.anims);
    print_data_map("Item", &bundle.items);
    print_data_map("Block", &bundle.blocks);
    print_data_map("Template", &bundle.templates);

    if let Some(ref world) = bundle.world {
        println!("\n\n == World ==");
        println!("\n{:#?}", world);
    }

    print_objects("Clients", &bundle.clients);
    print_objects("Entities", &bundle.entities);
    print_objects("Inventories", &bundle.inventories);
    print_objects("Planes", &bundle.planes);
    print_objects("Terrain Chunks", &bundle.terrain_chunks);
    print_objects("Structures", &bundle.structures);
}
