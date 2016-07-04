//! A cache for computed data about the world's terrain.  Most importantly, this includes the
//! computed shape of each block of terrain, derived from the `TerrainChunk` block at that position
//! and any structures that overlap it.  External callers notify the `TerrainCache` when something
//! changes in the world, so the cache can recompute the data for the relevant chunks.  Then other
//! engine parts (such as the physics engine) can query the cache for information about terrain.
use std::collections::HashMap;

use types::*;
use util::StrResult;
use libphysics::{CHUNK_BITS, CHUNK_SIZE};

use data::{Data, StructureTemplate};
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

    pub fn add_chunk(&mut self, data: &Data, pid: PlaneId, cpos: V2, blocks: &BlockChunk) {
        trace!("add chunk {:?} {:?}", pid, cpos);
        let mut entry = CacheEntry::new();
        entry.fill(data, blocks);
        self.cache.insert((pid, cpos), entry);
    }

    pub fn remove_chunk(&mut self, pid: PlaneId, cpos: V2) {
        trace!("remove chunk {:?} {:?}", pid, cpos);
        self.cache.remove(&(pid, cpos));
    }

    pub fn update_chunk(&mut self, data: &Data, pid: PlaneId, cpos: V2, blocks: &BlockChunk) {
        trace!("update chunk {:?} {:?}", pid, cpos);
        let entry = unwrap_or!(self.cache.get_mut(&(pid, cpos)));
        entry.refill(data, blocks);
    }

    pub fn add_structure(&mut self, pid: PlaneId, pos: V3, template: &StructureTemplate) {
        trace!("add structure {} at {:?} {:?}", template.name, pid, pos);
        let bounds = Region::sized(template.size) + pos;
        let bounds_chunk = bounds.reduce().div_round_signed(CHUNK_SIZE);

        for cpos in bounds_chunk.points() {
            let entry = unwrap_or!(self.cache.get_mut(&(pid, cpos)), continue);
            let base = (cpos * scalar(CHUNK_SIZE)).extend(0);
            let chunk_bounds = Region::sized(scalar(CHUNK_SIZE)) + base;
            for p in chunk_bounds.intersect(bounds).points() {
                let offset = p - base;
                let shape = template.shape[bounds.index(p)].shape();
                entry.set(offset, template.layer as i8, shape);
            }
        }
    }

    pub fn remove_structure(&mut self, pid: PlaneId, pos: V3, template: &StructureTemplate) {
        trace!("remove structure {} at {:?} {:?}", template.name, pid, pos);
        let bounds = Region::sized(template.size) + pos;
        let bounds_chunk = bounds.reduce().div_round_signed(CHUNK_SIZE);

        for cpos in bounds_chunk.points() {
            let entry = unwrap_or!(self.cache.get_mut(&(pid, cpos)), continue);
            let base = (cpos * scalar(CHUNK_SIZE)).extend(0);
            let chunk_bounds = Region::sized(scalar(CHUNK_SIZE)) + base;
            for p in chunk_bounds.intersect(bounds).points() {
                let offset = p - base;
                entry.set(offset, template.layer as i8, Shape::Empty);
            }
        }
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

    fn refill(&mut self, data: &Data, chunk: &BlockChunk) {
        let bounds = Region::sized(scalar(CHUNK_SIZE));
        for pos in bounds.points() {
            let old_shape = self.cells.get(&get_key(pos)).map_or(Shape::Empty, |c| c.base);
            let block = chunk[bounds.index(pos)];
            let new_shape = data.block_data.shape(block);
            if old_shape != new_shape {
                self.set(pos, -1, new_shape);
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
            if shape_overrides(cur, s) {
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
