/// Place actual ramps at each of the locations indicated in the summary.

extern crate rand;
extern crate rustc_serialize;

extern crate physics;
extern crate server_bundle;
extern crate server_config;
extern crate server_extra;
extern crate server_types;
#[macro_use] extern crate server_util;
extern crate server_world_types;
extern crate terrain_gen;
extern crate terrain_gen_algo;

extern crate save_0_6;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::iter;
use std::mem;
use std::path::Path;
use rand::XorShiftRng;
use rustc_serialize::json;

use physics::{CHUNK_SIZE, TILE_SIZE};
use server_bundle::flat;
use server_bundle::types::Bundle;
use server_config::{storage, data};
use server_extra as extra;
use server_extra::{Extra, View, ViewMut, Value};
use server_types::*;
use server_util::BitSlice;
use server_util::bytes::{ReadBytes, WriteBytes};
use server_world_types::{Motion, Item};
use terrain_gen::cache::Summary;
use terrain_gen::forest;
use terrain_gen::forest::common::GridLike;
use terrain_gen_algo::disk_sampler::DiskSampler;

use save_0_6::*;


fn load_bundle(path: &str) -> Bundle {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let flat = flat::FlatView::from_bytes(&buf).unwrap();
    flat.unflatten_bundle()
}

fn save_bundle(path: &str, b: &Bundle) {
    let mut f = File::create(path).unwrap();
    let mut flat = server_bundle::flat::Flat::new();
    flat.flatten_bundle(b);
    flat.write(&mut f).unwrap();
}


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

fn load_data(storage: &storage::Storage) -> data::Data {
    let block_json = read_json(storage.open_block_data());
    let item_json = read_json(storage.open_item_data());
    let recipe_json = read_json(storage.open_recipe_data());
    let template_json = read_json(storage.open_template_data());
    let animation_json = read_json(storage.open_animation_data());
    let sprite_part_json = read_json(storage.open_sprite_part_data());
    let loot_table_json = read_json(storage.open_loot_table_data());
    let data = data::Data::from_json(block_json,
                                     item_json,
                                     recipe_json,
                                     template_json,
                                     animation_json,
                                     sprite_part_json,
                                     loot_table_json).unwrap();

    data
}


fn load_preserved_set(path: &str) -> HashSet<V2> {
    let mut f = File::open(path).unwrap();
    let len = f.read_bytes::<u32>().unwrap() as usize;
    let mut result = HashSet::with_capacity(len);

    for _ in 0 .. len {
        result.insert(f.read_bytes().unwrap());
    }

    result
}

fn load_ramp_positions(dir: &str, cpos: V2) -> Vec<V3> {
    let path = format!("{}/save/summary/2015-12-05c-natural_ramps/2/{},{}",
                       dir, cpos.x, cpos.y);
    let mut f = File::open(&path).unwrap();
    let len = f.read_bytes::<u32>().unwrap() as usize;
    let mut buf = iter::repeat(scalar(0)).take(len).collect::<Vec<_>>();
    f.read_bytes_slice(&mut buf).unwrap();
    buf
}


fn place_ramp<F: FnMut(&str) -> BlockId>(blocks: &mut [BlockId],
                                         cave: &BitSlice,
                                         base: V3,
                                         mut get_id: F) {
    let cave_bounds = Region::<V2>::new(scalar(0), scalar(17));
    let block_bounds = Region::new(scalar(0), scalar(16));
    let get_cave = |cur: V3, dx, dy| {
        let pos = cur.reduce() + V2::new(dx, dy);
        if cave.get(cave_bounds.index(pos)) {
            2
        } else {
            0
        }
    };

    let gc = |x| { if x == 1 { 'g' } else { 'c' } };
    let z1 = V3::new(0, 0, 1);
    //let base = base - V3::new(3, 3, 0);

    let cur = base + V3::new(0, 0, 0);
    if block_bounds.contains(cur) {
        println!(" doing {:?}", cur);
        let nw = get_cave(cur, 0, 0);
        let ne = get_cave(cur, 1, 0);
        let sw = get_cave(cur, 0, 1);
        let se = 0;

        blocks[block_bounds.index(cur)] = get_id(
            &format!("ramp/xy00/z0/{}{}{}{}/c{}{}{}{}",
                     gc(nw), gc(ne), gc(se), gc(sw), nw, ne, se, sw));
        blocks[block_bounds.index(cur + z1)] = get_id(
            &format!("ramp/xy00/z1/c{}{}{}{}", nw, ne, se, sw));
    }

    let cur = base + V3::new(1, 0, 0);
    if block_bounds.contains(cur) {
        println!(" doing {:?}", cur);
        let nw = get_cave(cur, 0, 0);
        let ne = get_cave(cur, 1, 0);
        let sw = 0;
        let se = 0;

        blocks[block_bounds.index(cur)] = get_id(
            &format!("ramp/xy10/z0/{}{}{}{}/c{}{}{}{}",
                     gc(nw), gc(ne), gc(se), gc(sw), nw, ne, se, sw));
        blocks[block_bounds.index(cur + z1)] = get_id(
            &format!("ramp/xy10/z1/c{}{}{}{}", nw, ne, se, sw));
    }

    let cur = base + V3::new(2, 0, 0);
    if block_bounds.contains(cur) {
        println!(" doing {:?}", cur);
        let nw = get_cave(cur, 0, 0);
        let ne = get_cave(cur, 1, 0);
        let sw = 0;
        let se = get_cave(cur, 1, 1);

        blocks[block_bounds.index(cur)] = get_id(
            &format!("ramp/xy20/z0/{}{}{}{}/c{}{}{}{}",
                     gc(nw), gc(ne), gc(se), gc(sw), nw, ne, se, sw));
        blocks[block_bounds.index(cur + z1)] = get_id(
            &format!("ramp/xy20/z1/c{}{}{}{}", nw, ne, se, sw));
    }

    let cur = base + V3::new(0, 1, 0);
    if block_bounds.contains(cur) {
        println!(" doing {:?}", cur);
        let nw = get_cave(cur, 0, 0);
        let ne = 0;
        let sw = 1;
        let se = 1;

        blocks[block_bounds.index(cur)] = get_id(
            &format!("ramp/xy01/z0/{}{}{}{}/c{}{}{}{}",
                     gc(nw), gc(ne), gc(se), gc(sw), nw, ne, se, sw));
        blocks[block_bounds.index(cur + z1)] = get_id(
            &format!("ramp/xy01/z1/c{}{}{}{}", nw, ne, se, sw));
    }

    let cur = base + V3::new(2, 1, 0);
    if block_bounds.contains(cur) {
        println!(" doing {:?}", cur);
        let nw = 0;
        let ne = get_cave(cur, 1, 0);
        let sw = 1;
        let se = 1;

        blocks[block_bounds.index(cur)] = get_id(
            &format!("ramp/xy21/z0/{}{}{}{}/c{}{}{}{}",
                     gc(nw), gc(ne), gc(se), gc(sw), nw, ne, se, sw));
        blocks[block_bounds.index(cur + z1)] = get_id(
            &format!("ramp/xy21/z1/c{}{}{}{}", nw, ne, se, sw));
    }

    let cur = base + V3::new(1, 1, 1);
    if block_bounds.contains(cur) {
        blocks[block_bounds.index(cur)] = get_id("ramp/grass/z1");
    }

    let cur = base + V3::new(1, 2, 0);
    if block_bounds.contains(cur) {
        blocks[block_bounds.index(cur)] = get_id("ramp/grass/z0");
    }

    let cur = base + V3::new(1, 1, 2);
    if block_bounds.contains(cur) {
        blocks[block_bounds.index(cur)] = get_id("ramp/grass/cap1");
    }

    let cur = base + V3::new(1, 0, 2);
    if block_bounds.contains(cur) {
        blocks[block_bounds.index(cur)] = get_id("ramp/grass/cap0");
    }

}


fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir = &args[1];

    let storage = storage::Storage::new(dir);
    let data = load_data(&storage);

    let preserved = load_preserved_set("upgrade_preserved.dat");
    println!("found {} preserved chunks", preserved.len());

    let plane = load_bundle(&format!("{}/save/planes/2.plane", dir));
    let saved_chunks = plane.planes[0].saved_chunks.iter().map(|&x| x).collect::<HashMap<_,_>>();

    for &cpos in &preserved {
        let ramps = load_ramp_positions(dir, cpos);
        if ramps.len() == 0 {
            continue;
        }
        println!("found {} ramps in {:?}", ramps.len(), cpos);

        let mut bundle = load_bundle(&format!("{}/save/terrain_chunks/{:x}.terrain_chunk",
                                              dir, saved_chunks[&cpos].unwrap()));

        let mut blocks = bundle.blocks.iter().map(|x| x.to_owned()).collect::<Vec<_>>();
        let mut block_id_map = blocks.iter().enumerate().map(|(i,x)| (x.to_string(), i))
                                     .collect::<HashMap<_,_>>();
        {
            let mut get_block_id = |name: &str| {
                if let Some(&id) = block_id_map.get(name) {
                    id as BlockId
                } else {
                    println!("  alloc {}", name);
                    let id = blocks.len();
                    blocks.push(name.to_owned().into_boxed_str());
                    block_id_map.insert(name.to_owned(), id);
                    id as BlockId
                }
            };

            for &r in &ramps {
                println!("placing ramp at {:?}", r);
                let cave = {
                    use terrain_gen::forest::cave_detail::CaveDetail;
                    let f = File::open(format!("{}/save/summary/cave_detail/2/{},{}/{}",
                                               dir, cpos.x, cpos.y, r.z / 2)).unwrap();
                    CaveDetail::read_from(f).unwrap()
                };
                place_ramp(&mut *bundle.terrain_chunks[0].blocks,
                           cave.data(),
                           r,
                           |name| get_block_id(name));
            }
        }

        bundle.blocks = blocks.into_boxed_slice();

        save_bundle(&format!("{}/save/terrain_chunks/{:x}.terrain_chunk",
                             dir, saved_chunks[&cpos].unwrap()), &bundle);
    }
}
