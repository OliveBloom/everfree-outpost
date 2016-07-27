//! A cache for computed data about the world's terrain.  Most importantly, this includes the
//! computed shape of each block of terrain, derived from the `TerrainChunk` block at that position
//! and any structures that overlap it.  External callers notify the `TerrainCache` when something
//! changes in the world, so the cache can recompute the data for the relevant chunks.  Then other
//! engine parts (such as the physics engine) can query the cache for information about terrain.
use std::collections::HashMap;

use types::*;
use libphysics::{CHUNK_BITS, CHUNK_SIZE};

use data::{Data, StructureTemplate, BlockFlags};
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
                let flags = template.shape[bounds.index(p)];
                if flags.occupied() {
                    entry.set(offset, template.layer as i8, flags);
                }
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
                let flags = template.shape[bounds.index(p)];
                if flags.occupied() {
                    entry.set(offset, template.layer as i8, BlockFlags::empty());
                }
            }
        }
    }

    pub fn get(&self, pid: PlaneId, cpos: V2) -> Option<&CacheEntry> {
        self.cache.get(&(pid, cpos))
    }
}


pub struct CacheEntry {
    cells: HashMap<u16, CacheCell>,
}

impl CacheEntry {
    pub fn new() -> CacheEntry {
        CacheEntry {
            cells: HashMap::new(),
        }
    }

    fn fill(&mut self, data: &Data, chunk: &BlockChunk) {
        for (i, &block) in chunk.iter().enumerate() {
            let flags = data.block_data.block(block).flags;
            if !flags.is_empty() {
                let mut cell = CacheCell::new();
                cell.set_layer(-1, flags);
                self.cells.insert(i as u16, cell);
            }
        }
    }

    fn refill(&mut self, data: &Data, chunk: &BlockChunk) {
        let bounds = Region::sized(scalar(CHUNK_SIZE));
        for pos in bounds.points() {
            let old_flags = self.cells.get(&get_key(pos)).map_or(BlockFlags::empty(), |c| c.base);
            let block_id = chunk[bounds.index(pos)];
            let new_flags = data.block_data.block(block_id).flags;
            if old_flags != new_flags {
                self.set(pos, -1, new_flags);
            }
        }
    }

    pub fn get(&self, pos: V3) -> BlockFlags {
        if let Some(cell) = self.cells.get(&get_key(pos)) {
            cell.computed
        } else {
            BlockFlags::empty()
        }
    }

    fn set(&mut self, pos: V3, layer: i8, flags: BlockFlags) {
        use std::collections::hash_map::Entry;
        match self.cells.entry(get_key(pos)) {
            Entry::Vacant(e) => {
                if !flags.is_empty() {
                    let cell = e.insert(CacheCell::new());
                    cell.set_layer(layer, flags);
                }
            },
            Entry::Occupied(mut e) => {
                e.get_mut().set_layer(layer, flags);
                if e.get().computed.is_empty() {
                    e.remove();
                }
            },
        }
    }

    pub fn get_cell(&self, pos: V3) -> Option<&CacheCell> {
        self.cells.get(&get_key(pos))
    }
}


pub struct CacheCell {
    pub base: BlockFlags,
    pub layers: [BlockFlags; 3],
    pub computed: BlockFlags,
}

impl CacheCell {
    fn new() -> CacheCell {
        CacheCell {
            base: BlockFlags::empty(),
            layers: [BlockFlags::empty(); 3],
            computed: BlockFlags::empty(),
        }
    }

    fn set_layer(&mut self, layer: i8, flags: BlockFlags) {
        if layer == -1 {
            self.base = flags;
        } else {
            self.layers[layer as usize] = flags;
        }
        self.recompute();
    }

    fn recompute(&mut self) {
        let mut cur = self.base;
        for &s in &self.layers {
            // Note: this will break if multiple layers have a shape set under `B_SHAPE_MASK`.
            // So... don't do that.
            cur = cur | s;
        }
        self.computed = cur;
    }

    pub fn find_layer_with_flags(&self, flags: BlockFlags) -> Option<i8> {
        for i in (0 .. 3).rev() {
            if self.layers[i].contains(flags) {
                return Some(i as i8);
            }
        }
        if self.base.contains(flags) {
            return Some(-1);
        }
        None
    }
}


// Note that `fill` does its own key computation, instead of calling `get_key`.
fn get_key(pos: V3) -> u16 {
    ((pos.x as u16) << (0 * CHUNK_BITS)) |
    ((pos.y as u16) << (1 * CHUNK_BITS)) |
    ((pos.z as u16) << (2 * CHUNK_BITS))
}
