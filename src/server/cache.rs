//! A cache for computed data about the world's terrain.  Most importantly, this includes the
//! computed shape of each block of terrain, derived from the `TerrainChunk` block at that position
//! and any structures that overlap it.  External callers notify the `TerrainCache` when something
//! changes in the world, so the cache can recompute the data for the relevant chunks.  Then other
//! engine parts (such as the physics engine) can query the cache for information about terrain.
use std::collections::hash_map::{HashMap, Entry};

use types::*;
use libphysics::{CHUNK_BITS, CHUNK_SIZE};

use data::{Data, StructureTemplate, BlockFlags};


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
        self.with_entry(pid, cpos, |entry| {
            entry.fill_base(data, blocks);
        });
    }

    pub fn remove_chunk(&mut self, pid: PlaneId, cpos: V2) {
        trace!("remove chunk {:?} {:?}", pid, cpos);
        self.with_entry(pid, cpos, |entry| {
            entry.clear_base();
        });
    }

    pub fn update_chunk(&mut self, data: &Data, pid: PlaneId, cpos: V2, blocks: &BlockChunk) {
        trace!("update chunk {:?} {:?}", pid, cpos);
        self.with_entry(pid, cpos, |entry| {
            entry.fill_base(data, blocks);
        });
    }

    pub fn add_structure(&mut self, pid: PlaneId, pos: V3, template: &StructureTemplate) {
        trace!("add structure {} at {:?} {:?}", template.name, pid, pos);
        let bounds = Region::sized(template.size) + pos;
        let bounds_chunk = bounds.reduce().div_round_signed(CHUNK_SIZE);

        for cpos in bounds_chunk.points() {
            let base = (cpos * scalar(CHUNK_SIZE)).extend(0);
            let chunk_bounds = Region::sized(scalar(CHUNK_SIZE)) + base;
            self.with_entry(pid, cpos, |entry| {
                for p in chunk_bounds.intersect(bounds).points() {
                    let offset = p - base;
                    let flags = template.shape[bounds.index(p)];
                    if flags.occupied() {
                        entry.set(offset, template.layer as i8, flags);
                    }
                }
            });
        }
    }

    pub fn remove_structure(&mut self, pid: PlaneId, pos: V3, template: &StructureTemplate) {
        trace!("remove structure {} at {:?} {:?}", template.name, pid, pos);
        let bounds = Region::sized(template.size) + pos;
        let bounds_chunk = bounds.reduce().div_round_signed(CHUNK_SIZE);

        for cpos in bounds_chunk.points() {
            let base = (cpos * scalar(CHUNK_SIZE)).extend(0);
            let chunk_bounds = Region::sized(scalar(CHUNK_SIZE)) + base;
            self.with_entry(pid, cpos, |entry| {
                for p in chunk_bounds.intersect(bounds).points() {
                    let offset = p - base;
                    let flags = template.shape[bounds.index(p)];
                    if flags.occupied() {
                        entry.set(offset, template.layer as i8, BlockFlags::empty());
                    }
                }
            });
        }
    }

    pub fn get(&self, pid: PlaneId, cpos: V2) -> Option<&CacheEntry> {
        self.cache.get(&(pid, cpos))
    }

    fn with_entry<R, F: FnOnce(&mut CacheEntry) -> R>(&mut self,
                                                      pid: PlaneId,
                                                      cpos: V2,
                                                      f: F) -> R {
        match self.cache.entry((pid, cpos)) {
            Entry::Occupied(mut e) => {
                let r = f(e.get_mut());
                if e.get().cells.is_empty() {
                    trace!(" - chunk {:?} {:?} became empty", pid, cpos);
                    e.remove();
                }
                r
            },
            Entry::Vacant(e) => {
                let mut tmp = CacheEntry::new();
                let r = f(&mut tmp);
                if !tmp.cells.is_empty() {
                    trace!(" - chunk {:?} {:?} became nonempty", pid, cpos);
                    e.insert(tmp);
                }
                r
            },
        }
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

    fn fill_base(&mut self, data: &Data, chunk: &BlockChunk) {
        for (i, &block) in chunk.iter().enumerate() {
            let flags = data.block_data.block(block).flags;
            self.with_cell_key(i as u16, |cell| {
                if cell.base != flags {
                    cell.set_layer(-1, flags);
                }
            });
        }
    }

    fn clear_base(&mut self) {
        for i in 0 .. 1 << (3 * CHUNK_BITS) {
            self.with_cell_key(i, |cell| {
                cell.set_layer(-1, BlockFlags::empty());
            });
        }
    }

    fn with_cell_key<R, F: FnOnce(&mut CacheCell) -> R>(&mut self, k: u16, f: F) -> R {
        match self.cells.entry(k) {
            Entry::Occupied(mut e) => {
                let r = f(e.get_mut());
                if e.get().computed.is_empty() {
                    e.remove();
                }
                r
            },
            Entry::Vacant(e) => {
                let mut tmp = CacheCell::new();
                let r = f(&mut tmp);
                tmp.recompute();
                if !tmp.computed.is_empty() {
                    e.insert(tmp);
                }
                r
            },
        }
    }

    pub fn with_cell<R, F: FnOnce(&mut CacheCell) -> R>(&mut self, pos: V3, f: F) -> R {
        self.with_cell_key(get_key(pos), f)
    }

    pub fn get(&self, pos: V3) -> BlockFlags {
        if let Some(cell) = self.cells.get(&get_key(pos)) {
            cell.computed
        } else {
            BlockFlags::empty()
        }
    }

    pub fn set(&mut self, pos: V3, layer: i8, flags: BlockFlags) {
        self.with_cell(pos, |cell| {
            cell.set_layer(layer, flags);
        });
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
