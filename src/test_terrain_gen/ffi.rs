#![crate_name = "terrain_gen_ffi"]
#![crate_type = "staticlib"]

extern crate env_logger;
extern crate libc;
extern crate rand;
extern crate rustc_serialize;
extern crate terrain_gen as libterrain_gen;
extern crate server_config as libserver_config;
extern crate server_types as libserver_types;

use std::cmp;
use std::collections::hash_map;
use std::fs::File;
use std::ffi::CStr;
use std::io::Read;
use std::iter;
use std::mem;
use libc::{c_char, size_t};
use rand::{Rng, XorShiftRng, SeedableRng};
use rustc_serialize::json;

use libserver_config::{Data, Storage};
use libserver_types::*;
use libterrain_gen::{GenChunk, GenStructure};

use libterrain_gen::forest::Provider as ForestProvider;
use libterrain_gen::dungeon::Provider as DungeonProvider;

#[allow(dead_code)]
pub struct TerrainGen {
    data: Box<Data>,
    storage: Box<Storage>,
    forest: ForestProvider<'static>,
    dungeon: DungeonProvider<'static>,
}

fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

impl TerrainGen {
    fn new(path: &str) -> TerrainGen {
        let storage = Box::new(Storage::new(&path.to_owned()));

        let block_json = read_json(storage.open_block_data());
        let item_json = read_json(storage.open_item_data());
        let recipe_json = read_json(storage.open_recipe_data());
        let template_json = read_json(storage.open_template_data());
        let animation_json = read_json(storage.open_animation_data());
        let loot_table_json = read_json(storage.open_loot_table_data());
        let data = Box::new(Data::from_json(block_json,
                                            item_json,
                                            recipe_json,
                                            template_json,
                                            animation_json,
                                            loot_table_json).unwrap());

        let mut rng: XorShiftRng = SeedableRng::from_seed([0xe0e0e0e0,
                                                           0x00012345,
                                                           0xe0e0e0e0,
                                                           0x00012345]);
        // Cast away lifetimes so we can move `data`/`storage` into the struct later.
        let data_ref: &'static Data = unsafe { mem::transmute(&*data) };
        let storage_ref: &'static Storage = unsafe { mem::transmute(&*storage) };
        let forest = ForestProvider::new(data_ref, storage_ref, rng.gen());
        let dungeon = DungeonProvider::new(data_ref, storage_ref, rng.gen());

        TerrainGen {
            data: data,
            storage: storage,
            forest: forest,
            dungeon: dungeon,
        }
    }
}


static mut INITED_LOGGER: bool = false;

fn init_logger() {
    unsafe {
        if !INITED_LOGGER {
            env_logger::init().unwrap();
            INITED_LOGGER = true;
        }
    }
}


pub struct Drawing {
    size: V2,
    height_map: Box<[u8]>,
    points: Vec<(V2, &'static str)>,
    lines: Vec<(V2, V2, &'static str)>,
}

impl Drawing {
    pub fn new(size: V2) -> Drawing {
        let len = size.x as usize * size.y as usize;
        let height_map = iter::repeat(0).take(len).collect::<Vec<_>>().into_boxed_slice();
        Drawing {
            size: size,
            height_map: height_map,
            points: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn bounds(&self) -> Region<V2> {
        Region::new(scalar(0), self.size)
    }

    pub fn get_height(&self, pos: V2) -> u8 {
        self.height_map[self.bounds().index(pos)]
    }

    pub fn set_height(&mut self, pos: V2, val: u8) {
        self.height_map[self.bounds().index(pos)] = val;
    }

    pub fn add_point(&mut self, pos: V2, color: &'static str) {
        self.points.push((pos, color));
    }

    pub fn add_line(&mut self, pos0: V2, pos1: V2, color: &'static str) {
        self.lines.push((pos0, pos1, color));
    }
}


#[no_mangle]
pub unsafe extern "C" fn generator_create(path: *const c_char) -> *mut TerrainGen {
    init_logger();
    let c_str = CStr::from_ptr(path);
    let s = c_str.to_str().unwrap();
    let ptr = Box::new(TerrainGen::new(s));
    Box::into_raw(ptr)
}

#[no_mangle]
pub unsafe extern "C" fn generator_destroy(ptr: *mut TerrainGen) {
    drop(Box::from_raw(ptr));
}

#[no_mangle]
pub unsafe extern "C" fn generator_generate_chunk(ptr: *mut TerrainGen,
                                                  pid: u64,
                                                  x: i32,
                                                  y: i32) -> *mut GenChunk {
    let pid = Stable::new(pid);
    let cpos = V2::new(x, y);
    let chunk =
        if pid == STABLE_PLANE_FOREST {
            Box::new((*ptr).forest.generate(pid, cpos))
        } else {
            Box::new((*ptr).dungeon.generate(pid, cpos))
        };
    Box::into_raw(chunk)
}

#[no_mangle]
pub unsafe extern "C" fn generator_test(ptr: *mut TerrainGen,
                                        pid: u64,
                                        x: i32,
                                        y: i32) -> *mut Drawing {
    use libterrain_gen::forest::height_map;
    use libterrain_gen::forest::context::*;

    let pixel_size = 1;    // Number of tiles covered by each pixel
    let zoom = 1;
    let size = scalar(256 / zoom);
    let mut drawing = Box::new(Drawing::new(size));

    let pid = Stable::new(pid);
    let cpos = V2::new(x, y);
    let bounds = drawing.bounds() + cpos * size;

    /*
    (*ptr).forest.context_mut().grid_fold::<HeightMapPass, _, _>(
        pid, bounds, (), |(), pos, val| {
            //let val = cmp::max(0, cmp::min(255, val / 2 + 128)) as u8;
            let val =
                if val < -96 {
                    -1
                } else if val < 0 {
                    0
                } else if val < 256 {
                    (val / 32) as i8
                } else {
                    7
                };
            let val = (val + 1) as u8 * 31;
            drawing.height_map[bounds.index(pos)] = val;
        });
        // */

    (*ptr).forest.context_mut().grid_fold::<HeightDetailPass, _, _>(
        pid, bounds, (), |(), pos, val| {
            //let val = cmp::max(0, cmp::min(255, val / 2 + 128)) as u8;
            drawing.height_map[bounds.index(pos)] = (val + 1) as u8 * 31;
        });
        // */

    (*ptr).forest.context_mut().points_fold::<CaveRampsPass, _, _>(
        pid, bounds, (), |(), pos, r| {
            let h = drawing.height_map[bounds.index(pos)];
            if r.layer + 2 == h / 31 {
                drawing.add_point(pos - bounds.min, "blue");
            }
        });
        // */

    {
        let mut do_line = |x, y, color| {
            let x1 = x / pixel_size;
            let line_bounds = Region::new(V2::new(0, y), V2::new(x1, y + 1));
            if line_bounds.overlaps(bounds) {
                drawing.add_line(V2::new(0, y) - bounds.min,
                                 V2::new(x1, y) - bounds.min,
                                 color);
            }
        };

        do_line(8 * 16, 0, "red");
        do_line(160 * 16, 1, "green");
    }

    Box::into_raw(drawing)
}


#[no_mangle]
pub unsafe extern "C" fn chunk_free(ptr: *mut GenChunk) {
    drop(Box::from_raw(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn chunk_blocks_len(ptr: *const GenChunk) -> size_t {
    (*ptr).blocks.len() as size_t
}

#[no_mangle]
pub unsafe extern "C" fn chunk_get_block(ptr: *const GenChunk, idx: size_t) -> BlockId {
    (*ptr).blocks[idx as usize]
}

#[no_mangle]
pub unsafe extern "C" fn chunk_structures_len(ptr: *const GenChunk) -> size_t {
    (*ptr).structures.len() as size_t
}

#[no_mangle]
pub unsafe extern "C" fn chunk_get_structure(ptr: *const GenChunk,
                                             idx: size_t) -> *const GenStructure {
    &(*ptr).structures[idx as usize]
}


#[no_mangle]
pub unsafe extern "C" fn structure_get_pos(ptr: *const GenStructure,
                                           x_p: *mut i32,
                                           y_p: *mut i32,
                                           z_p: *mut i32) {
    *x_p = (*ptr).pos.x;
    *y_p = (*ptr).pos.y;
    *z_p = (*ptr).pos.z;
}

#[no_mangle]
pub unsafe extern "C" fn structure_get_template(ptr: *const GenStructure) -> TemplateId {
    (*ptr).template
}

#[no_mangle]
pub unsafe extern "C" fn structure_extra_len(ptr: *const GenStructure) -> size_t {
    (*ptr).extra.len() as size_t
}

#[no_mangle]
pub unsafe extern "C" fn structure_extra_iter(ptr: *const GenStructure) -> *mut ExtraIter {
    Box::into_raw(Box::new((*ptr).extra.iter()))
}


pub type ExtraIter = hash_map::Iter<'static, String, String>;

#[no_mangle]
pub unsafe extern "C" fn extra_iter_free(ptr: *mut ExtraIter) {
    drop(Box::from_raw(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn extra_iter_next(ptr: *mut ExtraIter,
                                         key_p: *mut *const c_char,
                                         key_len_p: *mut size_t,
                                         value_p: *mut *const c_char,
                                         value_len_p: *mut size_t) -> bool {
    match (*ptr).next() {
        None => false,
        Some((k, v)) => {
            *key_p = k.as_ptr() as *const c_char;
            *key_len_p = k.len() as size_t;
            *value_p = v.as_ptr() as *const c_char;
            *value_len_p = v.len() as size_t;
            true
        },
    }
}



#[no_mangle]
pub unsafe extern "C" fn drawing_free(ptr: *mut Drawing) {
    drop(Box::from_raw(ptr))
}

#[no_mangle]
pub unsafe extern "C" fn drawing_get_size(ptr: *mut Drawing,
                                          width_p: *mut u32,
                                          height_p: *mut u32) {
    *width_p = (*ptr).size.x as u32;
    *height_p = (*ptr).size.y as u32;
}

#[no_mangle]
pub unsafe extern "C" fn drawing_get_height_map(ptr: *mut Drawing) -> *const u8 {
    (*ptr).height_map.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn drawing_get_point_count(ptr: *mut Drawing) -> size_t {
    (*ptr).points.len()
}

#[no_mangle]
pub unsafe extern "C" fn drawing_get_point(ptr: *mut Drawing,
                                           i: size_t,
                                           x_p: *mut i32,
                                           y_p: *mut i32,
                                           color_p: *mut *const c_char,
                                           color_len_p: *mut size_t) {
    let &(pos, color) = &(*ptr).points[i];
    *x_p = pos.x;
    *y_p = pos.y;
    *color_p = color.as_ptr() as *const c_char;
    *color_len_p = color.len();
}

#[no_mangle]
pub unsafe extern "C" fn drawing_get_line_count(ptr: *mut Drawing) -> size_t {
    (*ptr).lines.len()
}

#[no_mangle]
pub unsafe extern "C" fn drawing_get_line(ptr: *mut Drawing,
                                          i: size_t,
                                          x0_p: *mut i32,
                                          y0_p: *mut i32,
                                          x1_p: *mut i32,
                                          y1_p: *mut i32,
                                          color_p: *mut *const c_char,
                                          color_len_p: *mut size_t) {
    let &(pos0, pos1, color) = &(*ptr).lines[i];
    *x0_p = pos0.x;
    *y0_p = pos0.y;
    *x1_p = pos1.x;
    *y1_p = pos1.y;
    *color_p = color.as_ptr() as *const c_char;
    *color_len_p = color.len();
}

