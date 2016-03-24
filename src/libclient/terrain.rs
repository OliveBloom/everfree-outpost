use std::mem;

use physics::v3::{V3, V2, Vn, scalar, Region};
use physics::{Shape, ShapeSource};
use physics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK};

pub const NUM_LAYERS: usize = 3;

pub type ShapeChunk = [Shape; 1 << (3 * CHUNK_BITS)];

pub struct ChunkShape {
    layers: [ShapeChunk; NUM_LAYERS],
    merged: ShapeChunk,
}

impl ChunkShape {
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
        self.refresh(inner_bounds);
    }

    fn find_ceiling(&self, pos: V3) -> i32 {
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
        for z in pos.z + 1 .. CHUNK_SIZE {
            if self.merged[chunk_bounds.index(pos.with_z(z))] != Shape::Empty {
                return z;
            }
        }
        CHUNK_SIZE
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
        unsafe { mem::zeroed() }
    }

    pub fn set_shape_in_region_by<F>(&mut self, bounds: Region, layer: usize, f: F)
            where F: Fn(V3) -> Shape {
        let cpos_bounds = bounds.reduce().div_round_signed(CHUNK_SIZE);
        let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE));
        for cpos in cpos_bounds.points() {
            let adj = (cpos * scalar(CHUNK_SIZE)).extend(0);
            let adj_bounds = bounds - adj;
            let cpos = cpos & scalar(LOCAL_MASK);
            self.chunks[local_bounds.index(cpos)].set_shape_in_region_by(
                adj_bounds, layer, |pos| f(pos + adj));
        }
    }

    pub fn set_shape_in_region(&mut self, bounds: Region, layer: usize, shape: &[Shape]) {
        self.set_shape_in_region_by(bounds, layer, |pos| shape[bounds.index(pos)]);
    }

    pub fn fill_shape_in_region(&mut self, bounds: Region, layer: usize, shape: Shape) {
        self.set_shape_in_region_by(bounds, layer, |_pos| shape);
    }

    pub fn find_ceiling(&self, pos: V3) -> i32 {
        let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE));
        let offset = pos - (cpos * scalar(CHUNK_SIZE)).extend(0);

        let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE));
        let cpos = cpos & scalar(LOCAL_MASK);

        self.chunks[local_bounds.index(cpos)].find_ceiling(offset)
    }
}
