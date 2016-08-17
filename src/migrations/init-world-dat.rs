/// Create an empty world.dat file.
///
/// Usage: ./init-world-dat output.dat

extern crate server_bundle;
extern crate server_extra;

use std::env;
use std::fs::File;
use server_bundle::types::{Bundle, World};
use server_bundle::flat::Flat;
use server_extra::Extra;


fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let path = &args[1];

    let world = World {
        now: 0,

        next_client: 0,
        next_entity: 0,
        next_inventory: 0,
        next_plane: 0,
        next_terrain_chunk: 0,
        next_structure: 0,

        extra: Extra::new(),
        child_entities: Vec::new().into_boxed_slice(),
        child_inventories: Vec::new().into_boxed_slice(),
    };

    let b = Bundle {
        anims: Vec::new().into_boxed_slice(),
        items: Vec::new().into_boxed_slice(),
        blocks: Vec::new().into_boxed_slice(),
        templates: Vec::new().into_boxed_slice(),

        world: Some(Box::new(world)),
        clients: Vec::new().into_boxed_slice(),
        entities: Vec::new().into_boxed_slice(),
        inventories: Vec::new().into_boxed_slice(),
        planes: Vec::new().into_boxed_slice(),
        terrain_chunks: Vec::new().into_boxed_slice(),
        structures: Vec::new().into_boxed_slice(),
    };

    let mut flat = Flat::new();
    flat.flatten_bundle(&b);
    flat.write(&mut File::create(&path).unwrap()).unwrap();
    println!("wrote output to {:?}", path);
}
