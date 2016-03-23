use std::prelude::v1::*;

use graphics::{lights, structures};
use graphics::types::{LocalChunks, BlockData, StructureTemplate, TemplatePart, TemplateVertex};

use physics;
use physics::CHUNK_BITS;
use physics::Shape;
use physics::v3::{V3, Region};

use data::Data;
use terrain::TerrainShape;
use terrain::LOCAL_BITS;


pub struct Client {
    data: Data,

    chunks: Box<LocalChunks>,
    terrain_shape: Box<TerrainShape>,
    structures: structures::Buffer,

    struct_geom_gen: structures::geom::GeomGenState,
    light_geom_gen: lights::GeomGenState,
}

impl Client {
    pub fn new(data: Data) -> Client {
        Client {
            data: data,

            chunks: box [[0; 1 << (3 * CHUNK_BITS)]; 1 << (2 * LOCAL_BITS)],
            terrain_shape: box TerrainShape::new(),
            structures: structures::Buffer::new(),

            struct_geom_gen: structures::geom::GeomGenState::new(Region::empty(), 0),
            light_geom_gen: lights::GeomGenState::new(Region::empty()),
        }
    }


    // Physics

    pub fn collide(&self, pos: V3, size: V3, velocity: V3) -> (V3, i32) {
        physics::collide(&*self.terrain_shape, pos, size, velocity)
    }

    pub fn set_region_shape(&mut self, bounds: Region, layer: usize, shape: &[Shape]) {
        self.terrain_shape.set_shape_in_region(bounds, layer, shape);
    }

    pub fn find_ceiling(&self, pos: V3) -> i32 {
        self.terrain_shape.find_ceiling(pos)
    }

    pub fn floodfill(&self,
                     pos: V3,
                     radius: u8,
                     grid: &mut [physics::fill_flags::Flags],
                     queue: &mut [(u8, u8)]) {
        physics::floodfill(pos, radius, &*self.terrain_shape, grid, queue);
    }
}
