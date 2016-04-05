use std::error::Error;
use std::ptr;
use std::slice;

use server_types::*;

use flat::FlatView;
use types::*;


/// Read a Bundle out of an array of bytes.
#[no_mangle]
pub unsafe extern "C" fn bundle_deserialize(ptr: *mut u8, len: usize) -> *mut Bundle {
    let bytes = slice::from_raw_parts(ptr, len);
    let f = match FlatView::from_bytes(bytes) {
        Ok(x) => x,
        Err(e) => {
            println!("bundle_deserialize: {}", e.description());
            return ptr::null_mut();
        },
    };
    let b = Box::new(f.unflatten_bundle());
    Box::into_raw(b)
}

/// Free a Bundle that was loaded with `bundle_deserialize`.
#[no_mangle]
pub unsafe extern "C" fn bundle_free(b: *mut Bundle) {
    drop(Box::from_raw(b))
}


#[no_mangle]
pub unsafe extern "C" fn bundle_blocks_len(b: *mut Bundle) -> usize {
    (*b).blocks.len()
}

#[no_mangle]
pub unsafe extern "C" fn bundle_blocks_get(b: *mut Bundle,
                                           index: usize,
                                           len: *mut usize) -> *const u8 {
    let s = &(*b).blocks;
    if index >= s.len() {
        return ptr::null();
    }
    if !len.is_null() {
        *len = s[index].len();
    }
    s[index].as_ptr() as *const u8
}

#[no_mangle]
pub unsafe extern "C" fn bundle_templates_len(b: *mut Bundle) -> usize {
    (*b).templates.len()
}

#[no_mangle]
pub unsafe extern "C" fn bundle_templates_get(b: *mut Bundle,
                                              index: usize,
                                              len: *mut usize) -> *const u8 {
    let s = &(*b).templates;
    if index >= s.len() {
        return ptr::null();
    }
    if !len.is_null() {
        *len = s[index].len();
    }
    s[index].as_ptr() as *const u8
}


#[no_mangle]
pub unsafe extern "C" fn bundle_planes_len(b: *mut Bundle) -> usize {
    (*b).planes.len()
}

#[no_mangle]
pub unsafe extern "C" fn bundle_planes_get(b: *mut Bundle,
                                           index: usize) -> *mut Plane {
    let s = &(*b).planes;
    if index >= s.len() {
        return ptr::null_mut();
    }
    &s[index] as *const _ as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn bundle_terrain_chunks_len(b: *mut Bundle) -> usize {
    (*b).terrain_chunks.len()
}

#[no_mangle]
pub unsafe extern "C" fn bundle_terrain_chunks_get(b: *mut Bundle,
                                                   index: usize) -> *mut TerrainChunk {
    let s = &(*b).terrain_chunks;
    if index >= s.len() {
        return ptr::null_mut();
    }
    &s[index] as *const _ as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn bundle_structures_len(b: *mut Bundle) -> usize {
    (*b).structures.len()
}

#[no_mangle]
pub unsafe extern "C" fn bundle_structures_get(b: *mut Bundle,
                                               index: usize) -> *mut Structure {
    let s = &(*b).structures;
    if index >= s.len() {
        return ptr::null_mut();
    }
    &s[index] as *const _ as *mut _
}


#[no_mangle]
pub unsafe extern "C" fn plane_saved_chunks_len(p: *mut Plane) -> usize {
    (*p).saved_chunks.len()
}

/// Get the saved chunk info at a given index.  Returns the `StableId` directly, and stores the
/// chunk position into `*x` and `*y`.
#[no_mangle]
pub unsafe extern "C" fn plane_saved_chunks_get(p: *mut Plane,
                                                index: usize,
                                                x: *mut i32,
                                                y: *mut i32) -> u64 {
    let s = &(*p).saved_chunks;
    if index >= s.len() {
        return 0;
    }
    if !x.is_null() {
        *x = s[index].0.x;
    }
    if !y.is_null() {
        *y = s[index].0.y;
    }
    s[index].1.unwrap()
}


#[no_mangle]
pub unsafe extern "C" fn terrain_chunk_blocks_len(tc: *mut TerrainChunk) -> usize {
    (*tc).blocks.len()
}

#[no_mangle]
pub unsafe extern "C" fn terrain_chunk_blocks_get(tc: *mut TerrainChunk,
                                                  index: usize) -> u16 {
    if index >= CHUNK_TOTAL {
        return 0;
    }
    (*tc).blocks[index]
}

#[no_mangle]
pub unsafe extern "C" fn terrain_chunk_child_structures_len(tc: *mut TerrainChunk) -> usize {
    (*tc).child_structures.len()
}

#[no_mangle]
pub unsafe extern "C" fn terrain_chunk_child_structures_get(tc: *mut TerrainChunk,
                                                            index: usize) -> u32 {
    let s = &(*tc).child_structures;
    if index >= s.len() {
        return 0;
    }
    s[index].unwrap()
}


/// Get the position of a `Structure`.  Stores the position into `*x`, `*y`, and `*z`.
#[no_mangle]
pub unsafe extern "C" fn structure_pos(s: *mut Structure,
                                       x: *mut i32,
                                       y: *mut i32,
                                       z: *mut i32) {
    *x = (*s).pos.x;
    *y = (*s).pos.y;
    *z = (*s).pos.z;
}

#[no_mangle]
pub unsafe extern "C" fn structure_template(s: *mut Structure) -> u32 {
    (*s).template
}
