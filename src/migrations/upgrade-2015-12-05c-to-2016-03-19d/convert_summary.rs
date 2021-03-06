/// Convert terrain gen summaries to the new format.

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


struct OldSummary {
    heightmap: [u8; 17 * 17],
    cave_entrances: Vec<V3>,
    natural_ramps: Vec<V3>,
    cave_walls: [[u8; (17 * 17 + 7) / 8]; 8],
    tree_offsets: Vec<V2>,
    treasure_offsets: [Vec<V2>; 8],
}

fn load_summary(path: &str) -> OldSummary {
    let mut f = File::open(path).unwrap();
    let mut summ: OldSummary = unsafe { mem::zeroed() };

    f.read_exact(&mut summ.heightmap).unwrap();

    // Skip heightmap_constraints
    let len = f.read_bytes::<u32>().unwrap() as usize;
    f.seek(SeekFrom::Current((len * mem::size_of::<(V2, (u8, u8))>()) as i64)).unwrap();

    let len = f.read_bytes::<u32>().unwrap() as usize;
    summ.cave_entrances = iter::repeat(scalar(0)).take(len).collect();
    f.read_bytes_slice(&mut summ.cave_entrances).unwrap();

    let len = f.read_bytes::<u32>().unwrap() as usize;
    summ.natural_ramps = iter::repeat(scalar(0)).take(len).collect();
    f.read_bytes_slice(&mut summ.natural_ramps).unwrap();

    for i in 0 .. 8 {
        f.read_exact(&mut summ.cave_walls[i]).unwrap();

        // Skip cave_wall_constraints[i]
        let len = f.read_bytes::<u32>().unwrap() as usize;
        f.seek(SeekFrom::Current((len * mem::size_of::<(V2, bool)>()) as i64)).unwrap();
    }

    let len = f.read_bytes::<u32>().unwrap() as usize;
    summ.tree_offsets = iter::repeat(scalar(0)).take(len).collect();
    f.read_bytes_slice(&mut summ.tree_offsets).unwrap();

    for i in 0 .. 8 {
        let len = f.read_bytes::<u32>().unwrap() as usize;
        summ.treasure_offsets[i] = iter::repeat(scalar(0)).take(len).collect();
        f.read_bytes_slice(&mut summ.treasure_offsets[i]).unwrap();
    }

    summ
}


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

fn make_adjacent_set(preserved: &HashSet<V2>) -> HashSet<V2> {
    let mut result = HashSet::new();

    for &cpos in preserved {
        for adj in Region::new(cpos - scalar(1), cpos + scalar(2)).points() {
            if !preserved.contains(&adj) {
                result.insert(adj);
            }
        }
    }

    result
}



fn conv_height(old: u8) -> i32 {
    let new = (old as i32 - 98) * 16 + 8;
    if new < -96 {
        // Don't allow generation of lakes from the old terrain.
        -96
    } else {
        new
    }
}

fn conv_height_detail(old: u8) -> i8 {
    if old < 98 {
        0
    } else if old >= 114 {
        7
    } else {
        (old as i8 - 98) / 2
    }
}

fn height_map_detail(h: i32) -> i8 {
    // NB: copied from forest/height_detail.rs
    if h < -96 {
        -1
    } else if h < 0 {
        0
    } else if h < 256 {
        // 0 .. 255 maps to 0 .. 7
        (h / 32) as i8
    } else {
        7
    }
}


fn update_height_map_cell(dir: &str, pos: V2, val: i32) {
    use terrain_gen::forest::height_map::HeightMap;

    let gpos = pos.div_floor(scalar(HeightMap::size()));
    let offset = pos - gpos * scalar(HeightMap::size());

    let mut grid = {
        let mut f = File::open(&format!("{}/save/summary/height_map/2/{},{}",
                                        dir, gpos.x, gpos.y)).unwrap();
        HeightMap::read_from(f).unwrap()
    };

    grid.data[HeightMap::bounds().index(offset)] = val;

    {
        let mut f = File::create(&format!("{}/save/summary/height_map/2/{},{}",
                                          dir, gpos.x, gpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}

fn get_height_map_cell(dir: &str, pos: V2) -> i32 {
    use terrain_gen::forest::height_map::HeightMap;

    let gpos = pos.div_floor(scalar(HeightMap::size()));
    let offset = pos - gpos * scalar(HeightMap::size());

    let mut grid = {
        let mut f = File::open(&format!("{}/save/summary/height_map/2/{},{}",
                                        dir, gpos.x, gpos.y)).unwrap();
        HeightMap::read_from(f).unwrap()
    };

    grid.data[HeightMap::bounds().index(offset)]
}



fn update_height_map(dir: &str, summ: &OldSummary, cpos: V2) {
    update_height_map_cell(dir, cpos + V2::new(0, 0), conv_height(summ.heightmap[0]));
    update_height_map_cell(dir, cpos + V2::new(1, 0), conv_height(summ.heightmap[16]));
    update_height_map_cell(dir, cpos + V2::new(0, 1), conv_height(summ.heightmap[17 * 16]));
    update_height_map_cell(dir, cpos + V2::new(1, 1), conv_height(summ.heightmap[17 * 16 + 16]));
}

fn update_height_detail(dir: &str, summ: &OldSummary, cpos: V2) {
    use terrain_gen::forest::height_detail::HeightDetail;

    let mut grid = HeightDetail::alloc();
    for i in 0 .. grid.data.len() {
        if cpos == V2::new(1, 0) {
            println!("  {}: map {} -> {}",
                     i, summ.heightmap[i],
                     conv_height_detail(summ.heightmap[i]));
        }
        grid.data[i] = conv_height_detail(summ.heightmap[i]);
    }

    {
        let mut f = File::create(&format!("{}/save/summary/height_detail/2/{},{}",
                                          dir, cpos.x, cpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}

fn update_cave_detail(dir: &str, summ: &OldSummary, cpos: V2, bundle: &Bundle) {
    use terrain_gen::forest::cave_detail::CaveDetail;

    let tc = &bundle.terrain_chunks[0];
    for layer in 0 .. 8 {
        let mut grid = {
            let f = File::open(&format!("{}/save/summary/cave_detail/2/{},{}/{}",
                                        dir, cpos.x, cpos.y, layer)).unwrap();
            CaveDetail::read_from(f).unwrap()
        };

        let old_grid = BitSlice::from_bytes(&summ.cave_walls[layer]);

        let bounds = Region::<V2>::new(scalar(0), scalar(17));
        for i in 0 .. grid.data().len() {
            grid.data_mut().set(i, old_grid.get(i));
        }

        let tile_bounds = Region::<V3>::new(scalar(0), scalar(16));
        let z = layer as i32 * 2;
        for pos in tile_bounds.reduce().points() {
            let block_id = tc.blocks[tile_bounds.index(pos.extend(z))];
            let block_name = &*bundle.blocks[block_id as usize];
            let last = block_name.split('/').next_back().unwrap();
            let mut chars = last.chars();
            if chars.next() != Some('c') {
                continue;
            }
            if let Some('2') = chars.next() {
                let idx = bounds.index(pos + V2::new(0, 0));
                grid.data_mut().set(idx, false);
            }
            if let Some('2') = chars.next() {
                let idx = bounds.index(pos + V2::new(1, 0));
                grid.data_mut().set(idx, false);
            }
            if let Some('2') = chars.next() {
                let idx = bounds.index(pos + V2::new(1, 1));
                grid.data_mut().set(idx, false);
            }
            if let Some('2') = chars.next() {
                let idx = bounds.index(pos + V2::new(0, 1));
                grid.data_mut().set(idx, false);
            }
        }

        {
            let mut f = File::create(&format!("{}/save/summary/cave_detail/2/{},{}/{}",
                                              dir, cpos.x, cpos.y, layer)).unwrap();
            grid.write_to(f).unwrap();
        }
    }
}

fn update_ramp_positions(dir: &str, cpos: V2) {
    use terrain_gen::forest::cave_ramps::{RampPositions, GRID_SIZE};

    let gpos = cpos.div_floor(scalar(GRID_SIZE / CHUNK_SIZE));
    let offset = cpos - gpos * scalar(GRID_SIZE / CHUNK_SIZE);

    let mut grid = {
        let f = File::open(&format!("{}/save/summary/cave_ramp_positions/2/{},{}",
                                    dir, gpos.x, gpos.y)).unwrap();
        RampPositions::read_from(f).unwrap()
    };

    let chunk_bounds = (Region::new(scalar(0), scalar(1)) + offset) * scalar(CHUNK_SIZE);

    let mut i = 0;
    while i < grid.data.len() {
        let bounds = Region::new(scalar(0), V2::new(3, 4)) + grid.data[i];
        if bounds.overlaps(chunk_bounds) {
            grid.data.swap_remove(i);
        } else {
            i += 1;
        }
    }


    {
        let mut f = File::create(&format!("{}/save/summary/cave_ramp_positions/2/{},{}",
                                          dir, gpos.x, gpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}

fn update_ramps(dir: &str, cpos: V2) {
    use terrain_gen::forest::cave_ramps::CaveRamps;

    // Write an empty list for this chunk
    let grid = CaveRamps::alloc();
    {
        let mut f = File::create(&format!("{}/save/summary/cave_ramps/2/{},{}",
                                          dir, cpos.x, cpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}

fn update_cave_junk(dir: &str, summ: &OldSummary, cpos: V2) {
    use terrain_gen::forest::cave_junk::CaveJunk;

    for layer in 0 .. 8 {
        let mut grid = CaveJunk::alloc();
        grid.data = summ.treasure_offsets[layer].clone();

        {
            let f = File::create(&format!("{}/save/summary/cave_junk/2/{},{}/{}",
                                          dir, cpos.x, cpos.y, layer)).unwrap();
            grid.write_to(f).unwrap();
        }
    }
}

fn update_natural_ramps(dir: &str, summ: &OldSummary, cpos: V2) {
    let mut f = File::create(&format!("{}/save/summary/2015-12-05c-natural_ramps/2/{},{}",
                                      dir, cpos.x, cpos.y)).unwrap();
    f.write_bytes(summ.natural_ramps.len() as u32).unwrap();

    for &r in &summ.natural_ramps {
        // Not sure why the offset is necessary, but it makes the ramps spawn in the right places.
        f.write_bytes(r - V3::new(3, 3, 0)).unwrap();
    }
}

fn update_cave_entrances(dir: &str, summ: &OldSummary, cpos: V2) {
    let mut f = File::create(&format!("{}/save/summary/2015-12-05c-cave_entrances/2/{},{}",
                                      dir, cpos.x, cpos.y)).unwrap();
    f.write_bytes(summ.cave_entrances.len() as u32).unwrap();
    f.write_bytes_slice(&summ.cave_entrances).unwrap();
}






fn load_height_detail(dir: &str, cpos: V2) -> Box<forest::height_detail::HeightDetail> {
    let mut f = File::open(&format!("{}/save/summary/height_detail/2/{},{}",
                                    dir, cpos.x, cpos.y)).unwrap();
    forest::height_detail::HeightDetail::read_from(f).unwrap()
}

fn create_blended_height_map(dir_path: &str, cpos: V2, preserved: &HashSet<V2>) {
    use terrain_gen::forest::height_detail::HeightDetail;

    let mut i32_grid = [0_i32; 17 * 17];
    let mut grid = HeightDetail::alloc();
    let bounds = HeightDetail::bounds();

    i32_grid[bounds.index(V2::new( 0,  0))] = get_height_map_cell(dir_path, cpos + V2::new(0, 0));
    i32_grid[bounds.index(V2::new(16,  0))] = get_height_map_cell(dir_path, cpos + V2::new(1, 0));
    i32_grid[bounds.index(V2::new( 0, 16))] = get_height_map_cell(dir_path, cpos + V2::new(0, 1));
    i32_grid[bounds.index(V2::new(16, 16))] = get_height_map_cell(dir_path, cpos + V2::new(1, 1));

    {
        let mut load_edge = |dir, base_in, base_out, step| {
            let adj_cpos = cpos + dir;
            if preserved.contains(&adj_cpos) {
                let grid = load_height_detail(dir_path, adj_cpos);
                for i in 1 .. 16 {
                    let in_pos = base_in + step * scalar(i);
                    let out_pos = base_out + step * scalar(i);
                    i32_grid[bounds.index(out_pos)] =
                        grid.data[bounds.index(in_pos)] as i32 * 32 + 16;
                }
            }
        };

        load_edge(V2::new(0, -1), V2::new(0, 16), V2::new(0,  0), V2::new(1, 0));
        load_edge(V2::new(0,  1), V2::new(0,  0), V2::new(0, 16), V2::new(1, 0));
        load_edge(V2::new(-1, 0), V2::new(16, 0), V2::new(0,  0), V2::new(0, 1));
        load_edge(V2::new( 1, 0), V2::new(0,  0), V2::new(16, 0), V2::new(0, 1));
    }

    {
        let mut fill_edge = |dir, base_out, step| {
            let adj_cpos = cpos + dir;
            if !preserved.contains(&adj_cpos) {
                let a = i32_grid[bounds.index(base_out)];
                let b = i32_grid[bounds.index(base_out + step * scalar(16))];
                for i in 1 .. 16 {
                    let out_pos = base_out + step * scalar(i);
                    i32_grid[bounds.index(out_pos)] = (a * (16 - i) + b * i) / 16;
                }
            }
        };

        fill_edge(V2::new(0, -1), V2::new(0,  0), V2::new(1, 0));
        fill_edge(V2::new(0,  1), V2::new(0, 16), V2::new(1, 0));
        fill_edge(V2::new(-1, 0), V2::new(0,  0), V2::new(0, 1));
        fill_edge(V2::new( 1, 0), V2::new(16, 0), V2::new(0, 1));
    }

    for pos in Region::<V2>::new(scalar(1), scalar(16)).points() {
        let n = i32_grid[bounds.index(V2::new(pos.x, 0))];
        let s = i32_grid[bounds.index(V2::new(pos.x, 16))];
        let w = i32_grid[bounds.index(V2::new(0, pos.y))];
        let e = i32_grid[bounds.index(V2::new(16, pos.y))];

        let h = (w * (16 - pos.x) + e * pos.x) / 16;
        let v = (n * (16 - pos.y) + s * pos.y) / 16;
        i32_grid[bounds.index(pos)] = (h + v) / 2;
    }

    for pos in bounds.points() {
        let idx = bounds.index(pos);
        grid.data[idx] = height_map_detail(i32_grid[idx]);
    }

    {
        let mut f = File::create(&format!("{}/save/summary/height_detail/2/{},{}",
                                          dir_path, cpos.x, cpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}



fn add_tree_positions(dir: &str, summ: &OldSummary, cpos: V2) {
    use terrain_gen::forest::trees::{TreePositions, GRID_SIZE};

    let gpos = cpos.div_floor(scalar(GRID_SIZE / CHUNK_SIZE));
    let offset = cpos - gpos * scalar(GRID_SIZE / CHUNK_SIZE);

    let path = format!("{}/save/summary/tree_positions/2/{},{}",
                       dir, gpos.x, gpos.y);
    let mut grid =
        if let Ok(f) = File::open(&path) {
            TreePositions::read_from(f).unwrap()
        } else {
            TreePositions::alloc()
        };

    for &pos in &summ.tree_offsets {
        grid.data.push(pos + offset * scalar(CHUNK_SIZE));
    }

    {
        let mut f = File::create(&format!("{}/save/summary/tree_positions/2/{},{}",
                                          dir, gpos.x, gpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}

pub fn generate_tree_positions(dir_path: &str, mut rng: XorShiftRng, gpos: V2) {
    use terrain_gen::forest::trees::{TreePositions, GRID_SIZE, SPACING};

    let mut disk = DiskSampler::new(scalar(3 * GRID_SIZE), SPACING, 2 * SPACING);
    let mut seen = HashSet::new();

    // Init disk samplers with points from adjacent chunks.
    for offset in Region::<V2>::new(scalar(-1), scalar(2)).points() {
        let adj_gpos = gpos + offset;
        let base = (offset + scalar(1)) * scalar(GRID_SIZE);
        if let Ok(f) = File::open(&format!("{}/save/summary/tree_positions/2/{},{}",
                                           dir_path, adj_gpos.x, adj_gpos.y)) {
            let grid = TreePositions::read_from(f).unwrap();
            for &pos in &grid.data {
                if !seen.contains(&(pos + base)) {
                    disk.add_init_point(pos + base);
                    seen.insert(pos + base);
                }
            }
        }
    }

    // Generate and save results
    disk.generate(&mut rng, 20);

    let mut grid = TreePositions::alloc();
    let base = scalar::<V2>(GRID_SIZE);
    let bounds = Region::new(base, base + scalar(GRID_SIZE));
    for &pos in disk.points() {
        if !bounds.contains(pos) {
            continue;
        }
        grid.data.push(pos - base);
    }

    {
        let mut f = File::create(&format!("{}/save/summary/tree_positions/2/{},{}",
                                          dir_path, gpos.x, gpos.y)).unwrap();
        grid.write_to(f).unwrap();
    }
}




fn empty_dir(dir: &str) {
    let mut paths = Vec::new();
    for ent in fs::read_dir(dir).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_file() {
            continue;
        }
        paths.push(ent.path());
    }

    for p in paths {
        fs::remove_file(p).unwrap();
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let old_dir = &args[1];
    let new_dir = &args[2];

    let storage = storage::Storage::new(new_dir);
    let data = load_data(&storage);

    let preserved = load_preserved_set("upgrade_preserved.dat");
    println!("found {} preserved chunks", preserved.len());
    let adjacent = make_adjacent_set(&preserved);
    println!("found {} adjacent chunks", adjacent.len());

    let plane = load_bundle(&format!("{}/save/planes/2.plane", new_dir));
    let saved_chunks = plane.planes[0].saved_chunks.iter().map(|&x| x).collect::<HashMap<_,_>>();


    let rng: XorShiftRng = rand::random();

    // Generate metadata for all preserved chunks.  Most of it will be overwritten or deleted
    // shortly, but we at least need the high-level heightmap containing each chunk.
    let mut provider = terrain_gen::forest::Provider::new(&data, &storage, rng.clone());
    for &cpos in &preserved {
        println!("  generating {:?}", cpos);
        // Generate, but drop the GenChunk.  This way it fills in save/summary, but doesn't
        // overwrite the .chunk.
        // TODO uncomment
        provider.generate(STABLE_PLANE_FOREST, cpos);
    }
    // Explicitly drop provider to ensure it writes all the metadata to disk.
    drop(provider);

    // Trees need special handling.  `trees` does not correspond directly to summ.tree_offsets.
    // Instead, we update `tree_positions`, and delete `trees` so it will be regenerated with
    // correct values.
    empty_dir(&format!("{}/save/summary/tree_positions/2", new_dir));
    empty_dir(&format!("{}/save/summary/trees/2", new_dir));

    let _ = fs::create_dir_all(&format!("{}/save/summary/2015-12-05c-natural_ramps/2/", new_dir));
    let _ = fs::create_dir_all(&format!("{}/save/summary/2015-12-05c-cave_entrances/2/", new_dir));

    // Write corrected metadata for each preserved chunk.
    for &cpos in &preserved {
        println!("processing {:?}", cpos);
        let summ = load_summary(&format!("{}/save/summary/2/chunk/{},{}.dat",
                                         old_dir, cpos.x, cpos.y));
        let chunk = load_bundle(&format!("{}/save/terrain_chunks/{:x}.terrain_chunk",
                                         new_dir, saved_chunks[&cpos].unwrap()));
        update_height_map(new_dir, &summ, cpos);
        update_height_detail(new_dir, &summ, cpos);
        update_cave_detail(new_dir, &summ, cpos, &chunk);
        update_ramp_positions(new_dir, cpos);
        update_ramps(new_dir, cpos);
        add_tree_positions(new_dir, &summ, cpos);
        update_cave_junk(new_dir, &summ, cpos);
        update_natural_ramps(new_dir, &summ, cpos);
        update_cave_entrances(new_dir, &summ, cpos);
    }

    // Finish tree handling
    let mut tree_chunks = HashSet::new();
    for &cpos in &preserved {
        use terrain_gen::forest::trees::GRID_SIZE;
        tree_chunks.insert(cpos.div_floor(scalar(GRID_SIZE / CHUNK_SIZE)));
    }
    println!("generating final tree positions for {} chunks", tree_chunks.len());
    for &gpos in &tree_chunks {
        println!("  - trees {:?}", gpos);
        generate_tree_positions(new_dir, rand::random(), gpos);
    }

    for &cpos in &adjacent {
        println!("create {:?}", cpos);
        create_blended_height_map(new_dir, cpos, &preserved);
    }
}
