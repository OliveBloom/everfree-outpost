/// Load all forest chunks and count the number of player-built structures.
///
/// Produce upgrade_survey.dat, containing the stable ID of each chunk containing at least one
/// artificial structure or terrain block.

extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;

extern crate save_0_6;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io;

use server_types::*;
use server_util::bytes::WriteBytes;

use save_0_6::*;

fn build_natural_structure_set() -> HashSet<String> {
    let mut h = HashSet::new();
    h.insert(String::from("tree/v0"));
    h.insert(String::from("tree/v1"));
    h.insert(String::from("stump"));
    h.insert(String::from("rock"));
    h.insert(String::from("cave_junk/0"));
    h.insert(String::from("cave_junk/1"));
    h.insert(String::from("cave_junk/2"));
    h.insert(String::from("chest"));
    h.insert(String::from("dungeon_entrance"));
    h
}

fn is_natural_block(name: &str) -> bool {
    name == "empty" ||
    name.starts_with("grass/center/v") ||
    name.starts_with("natural_ramp/") ||
    name.starts_with("cave/") ||
    name.starts_with("cave_top/")
}

fn load_plane(path: &str) -> Plane {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_plane().unwrap()
}

fn load_chunk(path: &str) -> io::Result<(TerrainChunk, HashMap<TemplateId, String>)> {
    let mut f = Reader::new(try!(File::open(path)));
    let h = try!(f.read_header());
    if (h.major, h.minor) != (0, 6) {
        fail!("version mismatch");
    }
    let tc = try!(f.read_terrain_chunk());
    Ok((tc, f.take_template_names()))
}

fn main() {
    let natural = build_natural_structure_set();

    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let plane = load_plane(&format!("{}/planes/2.plane", dir));

    let mut idx = 0;
    let count = plane.saved_chunks.len();
    let mut populated = Vec::new();
    for (&cpos, &id) in plane.saved_chunks.iter() {
        println!("loading tc {:5x} ({:5} / {:5})", id.unwrap(), idx, count);
        idx += 1;

        let path = format!("{}/terrain_chunks/{:x}.terrain_chunk",dir, id.unwrap());
        let (tc, names) = match load_chunk(&path) {
            Ok(x) => x,
            Err(e) => {
                println!("  error reading file: {:?}", e);
                continue;
            },
        };

        let first_artificial = names.iter()
                                    .filter(|&(_, name)| !natural.contains(name))
                                    .nth(0);
        if let Some((_, name)) = first_artificial {
            println!("  first artificial structure: {}", name);
            populated.push(id.unwrap());
            continue;
        }

        let first_artificial = tc.block_names.iter()
                                             .filter(|&(_, name)| !is_natural_block(name))
                                             .nth(0);
        if let Some((_, name)) = first_artificial {
            println!("  first artificial block: {}", name);
            populated.push(id.unwrap());
            continue;
        }
    }
    println!("found {} / {} populated chunks", populated.len(), count);

    let mut f = File::create("upgrade_survey.dat").unwrap();
    f.write_bytes(populated.len() as u32).unwrap();
    f.write_bytes_slice(&populated).unwrap();
}
