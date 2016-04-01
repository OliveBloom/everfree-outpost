/// Load all clients and retrieve their current and /home positions.
///
/// Produce upgrade_client_pos.dat, containing the cpos of each chunk that contains at least one
/// player or home position.

extern crate physics;
extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;

extern crate save_0_6;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::Path;

use physics::{CHUNK_SIZE, TILE_SIZE};
use server_extra::{View, Value};
use server_types::*;
use server_util::bytes::WriteBytes;

use save_0_6::*;

fn load_client(path: &Path) -> Client {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_client().unwrap()
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let mut positions = HashSet::new();
    for ent in fs::read_dir(&format!("{}/clients", dir)).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_file() {
            continue;
        }
        println!("loading {:?}", ent.path());
        let c = load_client(&ent.path());

        if c.child_entities.len() > 0 {
            let pos = c.child_entities[0].motion.start_pos;
            println!("  entity pos: {:?}", pos);
            let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
            positions.insert(cpos);
        }

        if let Some(View::Value(Value::V3(pos))) = c.extra.get("home_pos") {
            println!("  home pos: {:?}", pos);
            let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
            positions.insert(cpos);
        }
    }

    let pos_list = positions.iter().map(|&x| x).collect::<Vec<_>>();
    let mut f = File::create("upgrade_client_pos.dat").unwrap();
    f.write_bytes(pos_list.len() as u32).unwrap();
    f.write_bytes_slice(&pos_list).unwrap();
}
