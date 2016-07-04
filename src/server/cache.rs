//! A cache for computed data about the world's terrain.  Most importantly, this includes the
//! computed shape of each block of terrain, derived from the `TerrainChunk` block at that position
//! and any structures that overlap it.  External callers notify the `TerrainCache` when something
//! changes in the world, so the cache can recompute the data for the relevant chunks.  Then other
//! engine parts (such as the physics engine) can query the cache for information about terrain.
use std::collections::HashMap;

use types::*;
use util::StrResult;
use libphysics::{CHUNK_BITS, CHUNK_SIZE};

use data::Data;
use world::World;
use world::object::*;


pub struct TerrainCache {
    cache: HashMap<(PlaneId, V2), CacheEntry>,
}


impl TerrainCache {
    pub fn new() -> TerrainCache {
        TerrainCache {
            cache: HashMap::new(),
        }
    }

    pub fn add_chunk(&mut self, w: &World, pid: PlaneId, cpos: V2) -> StrResult<()> {
        let mut entry = CacheEntry::new();

        let data = w.data();
        let plane = unwrap!(w.get_plane(pid));
        let chunk = unwrap!(plane.get_terrain_chunk(cpos));
        entry.fill(data, chunk.blocks());

        self.cache.insert((pid, cpos), entry);
        Ok(())
    }

    pub fn remove_chunk(&mut self, pid: PlaneId, cpos: V2) {
        self.cache.remove(&(pid, cpos));
    }

    pub fn update_region(&mut self, w: &World, pid: PlaneId, bounds: Region) {
        // FIXME
        /*
        for cpos in bounds.reduce().div_round_signed(CHUNK_SIZE).points() {
            if let Some(entry) = self.cache.get_mut(&(pid, cpos)) {
                // NB: Surprisingly, this can fail.  Chunk unloading proceeds in this order:
                //  1) Remove terrain chunk
                //  2) Remove structures
                //  3) Run structure hooks
                //  4) Run terrain chunk hooks
                // During (3), the hook tries to update the cache.  The cache entry still exists
                // (because (4) hasn't happened yet), but the chunk is gone.
                let _ = compute_shape(w, pid, cpos, bounds, entry);
            }
        }
        */
    }

    pub fn get(&self, pid: PlaneId, cpos: V2) -> Option<&CacheEntry> {
        self.cache.get(&(pid, cpos))
    }
}


pub struct CacheEntry {
    cells: HashMap<u16, CellShape>,
}

impl CacheEntry {
    pub fn new() -> CacheEntry {
        CacheEntry {
            cells: HashMap::new(),
        }
    }

    fn fill(&mut self, data: &Data, chunk: &BlockChunk) {
        for (i, &block) in chunk.iter().enumerate() {
            let shape = data.block_data.shape(block);
            if shape != Shape::Empty {
                let mut cell = CellShape::new();
                cell.set_layer(-1, shape);
                self.cells.insert(i as u16, cell);
            }
        }
    }

    pub fn get(&self, pos: V3) -> Shape {
        if let Some(cell) = self.cells.get(&get_key(pos)) {
            cell.computed
        } else {
            Shape::Empty
        }
    }

    fn set(&mut self, pos: V3, layer: i8, shape: Shape) {
        use std::collections::hash_map::Entry;
        match self.cells.entry(get_key(pos)) {
            Entry::Vacant(e) => {
                if shape != Shape::Empty {
                    let cell = e.insert(CellShape::new());
                    cell.set_layer(layer, shape);
                }
            },
            Entry::Occupied(mut e) => {
                e.get_mut().set_layer(layer, shape);
                if e.get().computed == Shape::Empty {
                    e.remove();
                }
            },
        }
    }
}


struct CellShape {
    base: Shape,
    layers: [Shape; 3],
    computed: Shape,
}

impl CellShape {
    fn new() -> CellShape {
        CellShape {
            base: Shape::Empty,
            layers: [Shape::Empty; 3],
            computed: Shape::Empty,
        }
    }

    fn set_layer(&mut self, layer: i8, shape: Shape) {
        if layer == -1 {
            self.base = shape;
        } else {
            self.layers[layer as usize] = shape;
        }
        self.recompute();
    }

    fn recompute(&mut self) {
        let mut cur = self.base;
        for &s in &self.layers {
            if shape_overrides(s, cur) {
                cur = s;
            }
        }
        self.computed = cur;
    }
}


// Note that `fill` does its own key computation, instead of calling `get_key`.
fn get_key(pos: V3) -> u16 {
    ((pos.x as u16) << (0 * CHUNK_BITS)) |
    ((pos.y as u16) << (1 * CHUNK_BITS)) |
    ((pos.z as u16) << (2 * CHUNK_BITS))
}

fn shape_overrides(old: Shape, new: Shape) -> bool {
    match (old, new) {
        (Shape::Empty, _) => true,

        (Shape::Floor, Shape::Empty) => false,
        (Shape::Floor, _) => true,

        (Shape::Solid, _) => false,

        _ => false,
    }
}
