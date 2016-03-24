use std::prelude::v1::*;

use gl::GlContext;
use util;

use graphics::lights;
use graphics::types::{BlockChunk, LocalChunks,
    BlockData, StructureTemplate, TemplatePart, TemplateVertex};
use graphics::renderer::Renderer;

use physics;
use physics::{CHUNK_SIZE, CHUNK_BITS};
use physics::Shape;
use physics::v3::{V3, V2, scalar, Region};

use data::Data;
use structures::Structures;
use terrain::TerrainShape;
use terrain::{LOCAL_SIZE, LOCAL_BITS};


pub struct Client<GL: GlContext> {
    data: Data,

    chunks: Box<LocalChunks>,
    terrain_shape: Box<TerrainShape>,
    structures: Structures,

    renderer: Renderer<GL>,

    light_geom_gen: lights::GeomGenState,
}

impl<GL: GlContext> Client<GL> {
    pub fn new(data: Data, gl: GL) -> Client<GL> {
        Client {
            data: data,

            chunks: box [[0; 1 << (3 * CHUNK_BITS)]; 1 << (2 * LOCAL_BITS)],
            terrain_shape: box TerrainShape::new(),
            structures: Structures::new(),

            renderer: Renderer::new(gl),

            light_geom_gen: lights::GeomGenState::new(Region::empty()),
        }
    }


    pub fn load_terrain_chunk(&mut self, cpos: V2, blocks: &BlockChunk) {
        // Update self.chunks
        let bounds = Region::new(scalar(0), scalar(LOCAL_SIZE));
        self.chunks[bounds.index(cpos)] = *blocks;

        // Refresh self.terrain_shape
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE)) +
                           (cpos * scalar(CHUNK_SIZE)).extend(0);
        let block_data = &self.data.blocks;
        self.terrain_shape.set_shape_in_region_by(chunk_bounds, 0, |pos| {
            let b = blocks[chunk_bounds.index(pos)];
            block_data[b as usize].shape
        });

        // Invalidate cached geometry
        self.renderer.invalidate_terrain_geometry();
    }

    pub fn structure_appear(&mut self,
                            id: u32,
                            pos: (u8, u8, u8),
                            template_id: u32,
                            oneshot_start: u16) {
        // Update self.structures
        self.structures.insert(id, pos, template_id, oneshot_start);

        // Refresh self.terrain_cache
        let t = &self.data.templates[template_id as usize];
        let pos = util::unpack_v3(pos);
        let size = util::unpack_v3(t.size);
        let bounds = Region::new(pos, pos + size);
        let base = t.shape_idx as usize;
        let shape = &self.data.template_shapes[base .. base + bounds.volume() as usize];
        self.terrain_shape.set_shape_in_region(bounds, 1 + t.layer as usize, shape);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
    }

    pub fn structure_gone(&mut self,
                          id: u32) {
        let s = self.structures.remove(id);

        let t = &self.data.templates[s.template_id as usize];
        let pos = util::unpack_v3(s.pos);
        let size = util::unpack_v3(t.size);
        let bounds = Region::new(pos, pos + size);
        self.terrain_shape.fill_shape_in_region(bounds, 1 + t.layer as usize, Shape::Empty);
    }

    pub fn structure_replace(&mut self,
                             id: u32,
                             template_id: u32,
                             oneshot_start: u16) {
        self.structures.replace(id, template_id, oneshot_start);
        unimplemented!();
    }


    // Physics

    pub fn collide(&self, pos: V3, size: V3, velocity: V3) -> (V3, i32) {
        physics::collide(&*self.terrain_shape, pos, size, velocity)
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

    pub fn update_terrain_geometry(&mut self, bounds: Region<V2>) {
        self.renderer.update_terrain_geometry(&self.data, &self.chunks, bounds);
    }

    pub fn get_terrain_geometry_buffer(&self) -> &GL::Buffer {
        self.renderer.get_terrain_buffer()
    }

    pub fn update_structure_geometry(&mut self, bounds: Region<V2>) {
        self.renderer.update_structure_geometry(&self.data, &self.structures, bounds);
    }

    pub fn get_structure_geometry_buffer(&self) -> &GL::Buffer {
        self.renderer.get_structure_buffer()
    }

    pub fn light_geom_reset(&mut self, bounds: Region<V2>) {
        self.light_geom_gen = lights::GeomGenState::new(bounds);
    }

    pub fn light_geom_generate(&mut self,
                                   buf: &mut [lights::Vertex]) -> (usize, bool) {
        let mut gen = lights::GeomGen::new(&self.structures,
                                           &self.data.templates,
                                           &mut self.light_geom_gen);
        let mut idx = 0;
        let more = gen.generate(buf, &mut idx);

        (idx, more)
    }

}
