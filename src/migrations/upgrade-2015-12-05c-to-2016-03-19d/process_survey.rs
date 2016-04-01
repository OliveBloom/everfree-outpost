/// Process the chunk and client survey results to produce the complete list of preserved chunks.
///
/// From upgrade_survey.dat, add each entry to the preserved chunks.  Then, if the chunk contains a
/// ward, add the entire 5x5-chunk region surrounding the current chunk.
///
/// From upgrade_client_pos.dat, add each entry to the preserved chunks.
///
/// Produce upgrade_preserved.dat, containing the cpos of each preserved chunk.

extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;

extern crate save_0_6;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io;
use std::iter;

use server_types::*;
use server_util::bytes::{ReadBytes, WriteBytes};

use save_0_6::*;

fn load_plane(path: &str) -> Plane {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    f.read_plane().unwrap()
}

fn load_terrain_chunk(path: &str) -> (TerrainChunk, HashMap<TemplateId, String>) {
    let mut f = Reader::new(File::open(path).unwrap());
    let h = f.read_header().unwrap();
    assert!((h.major, h.minor) == (0, 6));
    let tc = f.read_terrain_chunk().unwrap();
    (tc, f.take_template_names())
}

fn chunk_exists(dir: &str, id: u64) -> bool {
    fs::metadata(&format!("{}/terrain_chunks/{:x}.terrain_chunk", dir, id)).is_ok()
}

fn main() {

    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let mut f = File::open("upgrade_survey.dat").unwrap();
    let len = f.read_bytes::<u32>().unwrap() as usize;
    let mut pop_ids = iter::repeat(0_u64).take(len).collect::<Vec<_>>();
    f.read_bytes_slice(&mut pop_ids);

    let mut f = File::open("upgrade_client_pos.dat").unwrap();
    let len = f.read_bytes::<u32>().unwrap() as usize;
    let mut client_pos = iter::repeat(scalar::<V2>(0)).take(len).collect::<Vec<_>>();
    f.read_bytes_slice(&mut client_pos);

    let p = load_plane(&format!("{}/planes/2.plane", dir));

    let mut cpos_map = HashMap::new();
    for (&cpos, &id) in &p.saved_chunks {
        cpos_map.insert(id.unwrap(), cpos);
    }

    let mut preserved_chunk_ids = HashSet::new();

    for &id in &pop_ids {
        preserved_chunk_ids.insert(id);

        let path = format!("{}/terrain_chunks/{:x}.terrain_chunk", dir, id);
        let (tc, names) = load_terrain_chunk(&path);

        if names.iter().any(|(_, name)| name == "ward") {
            let cpos = cpos_map[&id];
            println!("found ward in chunk: {:?} ({:x})", cpos, id);
            for new_cpos in Region::new(cpos - scalar(2), cpos + scalar(3)).points() {
                let tcid = match p.saved_chunks.get(&new_cpos) {
                    Some(x) => x.unwrap(),
                    None => continue,
                };

                if !chunk_exists(dir, tcid) {
                    continue;
                }

                println!("  add extra cpos: {:?} ({:x})", new_cpos, tcid);
                preserved_chunk_ids.insert(tcid);
            }
        }
    }

    for &cpos in &client_pos {
        let tcid = match p.saved_chunks.get(&cpos) {
            Some(x) => x.unwrap(),
            None => continue,
        };

        if !chunk_exists(dir, tcid) {
            continue;
        }

        println!("  add client cpos: {:?} ({:x})", cpos, tcid);
        preserved_chunk_ids.insert(tcid);
    }

    println!("{} total chunks preserved", preserved_chunk_ids.len());

    let preserved_list = preserved_chunk_ids.iter().map(|&x| cpos_map[&x]).collect::<Vec<_>>();
    let mut f = File::create("upgrade_preserved.dat").unwrap();
    f.write_bytes(preserved_list.len() as u32).unwrap();
    f.write_bytes_slice(&preserved_list).unwrap();


    /*
    let p = load_plane(&format!("{}/planes/2.plane", dir));
    let mut cpos_map = HashMap::new();
    for (&cpos, &id) in &p.saved_chunks {
        cpos_map.insert(id.unwrap(), cpos);
    }

    let mut cpos_list = Vec::new();
    for &id in &ids {
        cpos_list.push(cpos_map[&id]);
    }
    cpos_list.sort_by(|a, b| {
        (a.x, a.y).cmp(&(b.x, b.y))
    });
    for &cpos in &cpos_list {
        println!("{:?}", cpos);
    }
    */
}
