use std::prelude::v1::*;

use graphics::{lights, structures};
use graphics::terrain as g_terrain;
use graphics::types::{BlockChunk, LocalChunks,
    BlockData, StructureTemplate, TemplatePart, TemplateVertex};

use physics;
use physics::CHUNK_BITS;
use physics::Shape;
use physics::v3::{V3, V2, scalar, Region};

use data::Data;
use terrain::TerrainShape;
use terrain::{LOCAL_SIZE, LOCAL_BITS};


pub struct Client {
    data: Data,

    chunks: Box<LocalChunks>,
    terrain_shape: Box<TerrainShape>,
    structures: structures::Buffer,

    terrain_geom_gen: g_terrain::GeomGenState,
    structure_geom_gen: structures::geom::GeomGenState,
    light_geom_gen: lights::GeomGenState,
}

impl Client {
    pub fn new(data: Data) -> Client {
        Client {
            data: data,

            chunks: box [[0; 1 << (3 * CHUNK_BITS)]; 1 << (2 * LOCAL_BITS)],
            terrain_shape: box TerrainShape::new(),
            structures: structures::Buffer::new(),

            terrain_geom_gen: g_terrain::GeomGenState::new(scalar(0)),
            structure_geom_gen: structures::geom::GeomGenState::new(Region::empty(), 0),
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


    // Graphics

    pub fn load_terrain_chunk(&mut self, cpos: V2, blocks: &BlockChunk) {
        let bounds = Region::new(scalar(0), scalar(LOCAL_SIZE));
        self.chunks[bounds.index(cpos)] = *blocks;
    }

    pub fn terrain_geom_reset(&mut self, cpos: V2) {
        self.terrain_geom_gen = g_terrain::GeomGenState::new(cpos);
    }

    pub fn terrain_geom_generate(&mut self, buf: &mut [g_terrain::Vertex]) -> (usize, bool) {
        let mut gen = g_terrain::GeomGen::new(&self.chunks,
                                              &self.data.block_data,
                                              &mut self.terrain_geom_gen);
        let mut idx = 0;
        let more = gen.generate(buf, &mut idx);

        (idx, more)
    }

    pub fn structure_buffer_insert(&mut self,
                                   external_id: u32,
                                   pos: (u8, u8, u8),
                                   template_id: u32,
                                   oneshot_start: u16) -> usize {
        self.structures.insert(external_id, pos, template_id, oneshot_start)
    }

    pub fn structure_buffer_remove(&mut self,
                                   idx: usize) -> u32 {
        self.structures.remove(idx)
    }

    pub fn structure_geom_reset(&mut self, bounds: Region<V2>, sheet: u8) {
        self.structure_geom_gen = structures::geom::GeomGenState::new(bounds, sheet);
    }

    pub fn structure_geom_generate(&mut self,
                                   buf: &mut [structures::geom::Vertex]) -> (usize, bool) {
        println!("doing geometry for {} structures", self.structures.len());
        let mut gen = structures::geom::GeomGen::new(&self.structures,
                                                     &self.data.template_data,
                                                     &mut self.structure_geom_gen);
        let mut idx = 0;
        let more = gen.generate(buf, &mut idx);
        println!("  generated {}, more={}", idx, more);

        (idx, more)
    }

    pub fn light_geom_reset(&mut self, bounds: Region<V2>) {
        self.light_geom_gen = lights::GeomGenState::new(bounds);
    }

    pub fn light_geom_generate(&mut self,
                                   buf: &mut [lights::Vertex]) -> (usize, bool) {
        let mut gen = lights::GeomGen::new(&self.structures,
                                           &self.data.template_data.templates,
                                           &mut self.light_geom_gen);
        let mut idx = 0;
        let more = gen.generate(buf, &mut idx);

        (idx, more)
    }

}
