use types::*;
use std::mem;

use physics::v3::{V3, V2, scalar, Region};
use physics::ShapeSource;
use physics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK};

use data::Data;
use structures::Structures;


pub const NUM_LAYERS: usize = 4;

pub type ShapeChunk = [Shape; 1 << (3 * CHUNK_BITS)];

pub struct ChunkShape {
    layers: [ShapeChunk; NUM_LAYERS],
    merged: ShapeChunk,
}

impl ChunkShape {
    fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            for shape in layer.iter_mut() {
                *shape = Shape::Empty;
            }
        }

        for shape in self.merged.iter_mut() {
            *shape = Shape::Empty;
        }
    }

    fn reset_structure_layers(&mut self) {
        for layer in self.layers.iter_mut().skip(1) {
            for shape in layer.iter_mut() {
                *shape = Shape::Empty;
            }
        }
    }

    fn refresh(&mut self, bounds: Region) {
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));

        for p in bounds.intersect(chunk_bounds).points() {
            let idx = chunk_bounds.index(p);
            self.merged[idx] = self.layers[0][idx];

            for layer in self.layers.iter().skip(1) {
                if shape_overrides(self.merged[idx], layer[idx]) {
                    self.merged[idx] = layer[idx];
                }
            }
        }
    }

    fn set_shape_in_region_by<F>(&mut self, bounds: Region, layer: usize, f: F)
            where F: Fn(V3) -> Shape {
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
        let inner_bounds = bounds.intersect(chunk_bounds);
        for pos in inner_bounds.points() {
            self.layers[layer][chunk_bounds.index(pos)] = f(pos);
        }
    }
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


pub const LOCAL_BITS: usize = 3;
pub const LOCAL_SIZE: i32 = 1 << LOCAL_BITS;    // 8
pub const LOCAL_MASK: i32 = LOCAL_SIZE - 1;

pub struct TerrainShape {
    chunks: [ChunkShape; 1 << (2 * LOCAL_BITS)],
    structures_valid: [bool; 1 << (2 * LOCAL_BITS)],
    any_structures_invalid: bool,
}

impl ShapeSource for TerrainShape {
    fn get_shape(&self, pos: V3) -> Shape {
        if pos.z < 0 || pos.z >= CHUNK_SIZE {
            return Shape::Solid;
        }

        let tile = pos & scalar(CHUNK_MASK);
        let chunk = (pos.reduce() >> CHUNK_BITS) & scalar(LOCAL_MASK);

        let local_bounds = Region::<V2>::new(scalar(0), scalar(LOCAL_SIZE));
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));

        self.chunks[local_bounds.index(chunk)].merged[chunk_bounds.index(tile)]
    }
}

impl TerrainShape {
    pub fn new() -> TerrainShape {
        // 0 == Shape::Empty
        TerrainShape {
            chunks: unsafe { mem::zeroed() },
            structures_valid: [true; 1 << (2 * LOCAL_BITS)],
            any_structures_invalid: false,
        }
    }

    pub fn clear(&mut self) {
        for chunk in self.chunks.iter_mut() {
            chunk.clear();
        }
    }

    pub fn refresh_structures(&mut self, structures: &Structures, data: &Data) {
        if !self.any_structures_invalid {
            return;
        }

        let local_bounds = Region::<V2>::new(scalar(0), scalar(LOCAL_SIZE));
        let chunk_bounds = Region::sized(scalar(CHUNK_SIZE));

        for cpos in local_bounds.points() {
            let idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            if !self.structures_valid[idx] {
                self.chunks[local_bounds.index(cpos)].reset_structure_layers();
            }
        }

        for (_, s) in structures.iter() {
            self.add_structure_impl(data, s.pos(), s.template_id, true);
        }

        for cpos in local_bounds.points() {
            let idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            if !self.structures_valid[idx] {
                self.chunks[idx].refresh(chunk_bounds);
                self.structures_valid[idx] = true;
            }
        }
        self.any_structures_invalid = false;
    }

    pub fn set_terrain(&mut self, data: &Data, cpos: V2, blocks: &[BlockId]) {
        let local_bounds = Region::sized(scalar(LOCAL_SIZE));
        let chunk_bounds = Region::sized(scalar(CHUNK_SIZE));
        let idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
        self.chunks[idx].set_shape_in_region_by(chunk_bounds, 0, |pos| {
            let block = blocks[chunk_bounds.index(pos)];
            data.block(block).flags().shape()
        });
        self.chunks[idx].refresh(chunk_bounds);
    }

    fn add_structure_impl(&mut self,
                          data: &Data,
                          pos: V3,
                          template_id: TemplateId,
                          refreshing: bool) {
        let t = data.template(template_id);
        let shape = data.template_shape(template_id);

        let bounds = Region::sized(t.size()) + pos;
        let cpos_bounds = bounds.reduce().div_round_signed(CHUNK_SIZE);
        let local_bounds = Region::sized(scalar(LOCAL_SIZE));

        for cpos in cpos_bounds.points() {
            let idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            if !refreshing {
                if !self.structures_valid[idx] { continue; }
            } else {
                if self.structures_valid[idx] { continue; }
            }

            let adj = (cpos * scalar(CHUNK_SIZE)).extend(0);
            self.chunks[idx].set_shape_in_region_by(bounds - adj, 1 + t.layer as usize, |pos| {
                shape[bounds.index(pos + adj)].shape()
            });
            if !refreshing {
                self.chunks[idx].refresh(bounds - adj);
            }
        }
    }

    pub fn add_structure(&mut self, data: &Data, pos: V3, template_id: TemplateId) {
        self.add_structure_impl(data, pos, template_id, false);
    }

    pub fn remove_structure(&mut self, data: &Data, pos: V3, template_id: TemplateId) {
        let t = data.template(template_id);

        let bounds = Region::sized(t.size()) + pos;
        let cpos_bounds = bounds.reduce().div_round_signed(CHUNK_SIZE);
        let local_bounds = Region::sized(scalar(LOCAL_SIZE));

        for cpos in cpos_bounds.points() {
            let idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
            self.structures_valid[idx] = false;
            self.any_structures_invalid = true;
        }
    }
}
