/// Create an empty plane file.
///
/// Usage: ./init-plane output.plane name stable_id

extern crate server_bundle;
extern crate server_extra;

use std::env;
use std::fs::File;
use std::str::FromStr;
use server_bundle::types::{Bundle, Plane};
use server_bundle::flat::Flat;
use server_extra::Extra;


fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let path = &args[1];
    let name = &args[2];
    let stable_id = u64::from_str(&args[3]).unwrap();

    let plane = Plane {
        name: name.to_owned().into_boxed_str(),

        saved_chunks: Vec::new().into_boxed_slice(),

        extra: Extra::new(),
        stable_id: stable_id,
    };

    let b = Bundle {
        anims: Vec::new().into_boxed_slice(),
        items: Vec::new().into_boxed_slice(),
        blocks: Vec::new().into_boxed_slice(),
        templates: Vec::new().into_boxed_slice(),

        world: None,
        clients: Vec::new().into_boxed_slice(),
        entities: Vec::new().into_boxed_slice(),
        inventories: Vec::new().into_boxed_slice(),
        planes: vec![plane].into_boxed_slice(),
        terrain_chunks: Vec::new().into_boxed_slice(),
        structures: Vec::new().into_boxed_slice(),
    };

    let mut flat = Flat::new();
    flat.flatten_bundle(&b);
    flat.write(&mut File::create(&path).unwrap()).unwrap();
    println!("wrote output to {:?}", path);
}
