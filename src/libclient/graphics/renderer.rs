use std::prelude::v1::*;
use std::mem;
use std::slice;

use physics::v3::{V2, scalar, Region};

use data::Data;
use entity::Entities;
use graphics;
use graphics::GeometryGenerator;
use graphics::types::LocalChunks;
use platform::gl::{Context, Buffer, Framebuffer};
use platform::gl::{DrawArgs, UniformValue, DepthBuffer};
use structures::Structures;
use terrain::{LOCAL_SIZE, LOCAL_MASK};
use ui;
use util;

use super::entity;
use super::light;
use super::structure;
use super::terrain;


struct Buffers<GL: Context> {
    square01: GL::Buffer,
}

impl<GL: Context> Buffers<GL> {
    fn new(gl: &mut GL) -> Buffers<GL> {
        Buffers {
            square01: gl.create_buffer_with_data(&[
                0, 0,
                0, 1,
                1, 1,

                0, 0,
                1, 1,
                1, 0,
            ]),
        }
    }
}

struct Shaders<GL: Context> {
    blit_full: GL::Shader,
    terrain: GL::Shader,
    structure: GL::Shader,
    structure_shadow: GL::Shader,
    light_static: GL::Shader,
}

impl<GL: Context> Shaders<GL> {
    fn new(gl: &mut GL) -> Shaders<GL> {
        Shaders {
            blit_full: gl.load_shader(
                "blit_fullscreen.vert", "blit_output.frag", "",
                uniforms! {},
                arrays! {
                    [2] attribs! {
                        posOffset: U8[2] @0,
                    },
                },
                textures! { imageTex, },
                outputs! { color: 1 }),

            terrain: terrain::load_shader(gl),

            structure: structure::load_shader(gl, false),
            structure_shadow: structure::load_shader(gl, true),

            light_static: light::load_shader(gl),
        }
    }
}

struct Textures<GL: Context> {
    tile_atlas: GL::Texture,
    structure_atlas: GL::Texture,
}

impl<GL: Context> Textures<GL> {
    fn new(gl: &mut GL) -> Textures<GL> {
        Textures {
            tile_atlas: gl.load_texture("tiles"),
            structure_atlas: gl.load_texture("structures0"),
        }
    }
}

struct Framebuffers<GL: Context> {
    world: GL::Framebuffer,
}

impl<GL: Context> Framebuffers<GL> {
    fn new(gl: &mut GL, screen_size: (u16, u16)) -> Framebuffers<GL> {
        Framebuffers {
            world: gl.create_framebuffer(screen_size, 2, DepthBuffer::Texture),
        }
    }
}

pub struct Renderer<GL: Context> {
    terrain_geom: GeomCache<GL, Region<V2>>,
    structure_geom: GeomCache<GL, Region<V2>>,
    light_geom: GeomCache<GL, Region<V2>>,
    entity_buffer: GL::Buffer,
    ui_buffer: GL::Buffer,

    buffers: Buffers<GL>,
    shaders: Shaders<GL>,
    textures: Textures<GL>,
    framebuffers: Framebuffers<GL>,
}

impl<GL: Context> Renderer<GL> {
    pub fn new(gl: &mut GL) -> Renderer<GL> {
        Renderer {
            terrain_geom: GeomCache::new(gl),
            structure_geom: GeomCache::new(gl),
            light_geom: GeomCache::new(gl),
            entity_buffer: gl.create_buffer(),
            ui_buffer: gl.create_buffer(),

            buffers: Buffers::new(gl),
            shaders: Shaders::new(gl),
            textures: Textures::new(gl),
            framebuffers: Framebuffers::new(gl, (640, 480)),
        }
    }


    pub fn update_terrain_geometry(&mut self,
                                   data: &Data,
                                   chunks: &LocalChunks,
                                   bounds: Region<V2>) {
        self.terrain_geom.update(bounds, |buffer, _| {
            let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE as i32));

            let mut vert_count = 0;
            for cpos in bounds.points() {
                let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
                vert_count += terrain::GeomGen::new(&data.blocks,
                                                    &chunks[chunk_idx],
                                                    cpos).count_verts();
            }

            buffer.alloc(vert_count * mem::size_of::<terrain::Vertex>());
            let mut tmp = unsafe { util::zeroed_boxed_slice(64 * 1024) };

            let mut offset = 0;
            for cpos in bounds.points() {
                let chunk_idx = local_bounds.index(cpos & scalar(LOCAL_MASK));
                let mut gen = terrain::GeomGen::new(&data.blocks,
                                                    &chunks[chunk_idx],
                                                    cpos);
                load_buffer::<GL, _>(buffer, &mut gen, &mut tmp, &mut offset);
            }
        });
    }

    pub fn invalidate_terrain_geometry(&mut self) {
        self.terrain_geom.invalidate();
    }

    pub fn get_terrain_buffer(&self) -> &GL::Buffer {
        self.terrain_geom.buffer()
    }


    pub fn update_structure_geometry(&mut self,
                                     data: &Data,
                                     structures: &Structures,
                                     bounds: Region<V2>) {
        self.structure_geom.update(bounds, |buffer, _| {
            let mut gen = structure::GeomGen::new(structures, data, bounds);
            buffer.alloc(gen.count_verts() * mem::size_of::<structure::Vertex>());
            let mut tmp = unsafe { util::zeroed_boxed_slice(64 * 1024) };
            load_buffer::<GL, _>(buffer, &mut gen, &mut tmp, &mut 0);
        });
    }

    pub fn invalidate_structure_geometry(&mut self) {
        self.structure_geom.invalidate();
    }

    pub fn get_structure_buffer(&self) -> &GL::Buffer {
        self.structure_geom.buffer()
    }


    pub fn update_light_geometry(&mut self,
                                 data: &Data,
                                 structures: &Structures,
                                 bounds: Region<V2>) {
        self.light_geom.update(bounds, |buffer, _| {
            let mut gen = light::GeomGen::new(structures, &data.templates, bounds);
            buffer.alloc(gen.count_verts() * mem::size_of::<light::Vertex>());
            let mut tmp = unsafe { util::zeroed_boxed_slice(64 * 1024) };
            load_buffer::<GL, _>(buffer, &mut gen, &mut tmp, &mut 0);
        });
    }

    pub fn invalidate_light_geometry(&mut self) {
        self.light_geom.invalidate();
    }

    pub fn get_light_buffer(&self) -> &GL::Buffer {
        self.light_geom.buffer()
    }


    pub fn load_ui_geometry(&mut self, geom: &[ui::geom::Vertex]) {
        let byte_len = geom.len() * mem::size_of::<ui::geom::Vertex>();
        let bytes = unsafe {
            slice::from_raw_parts(geom.as_ptr() as *const u8, byte_len)
        };

        self.ui_buffer.alloc(byte_len);
        self.ui_buffer.load(0, bytes);
    }

    pub fn get_ui_buffer(&self) -> &GL::Buffer {
        &self.ui_buffer
    }


    pub fn update_entity_geometry(&mut self,
                                  data: &Data,
                                  entities: &Entities,
                                  bounds: Region<V2>,
                                  now: i32) {
        let mut gen = entity::GeomGen::new(entities, data, bounds, now);
        self.entity_buffer.alloc(gen.count_verts() * mem::size_of::<entity::Vertex>());
        let mut tmp = unsafe { util::zeroed_boxed_slice(64 * 1024) };
        load_buffer::<GL, _>(&mut self.entity_buffer, &mut gen, &mut tmp, &mut 0);
    }


    pub fn render_terrain(&mut self, scene: &Scene, cavern_tex: &GL::Texture) {
        let terrain_buf = self.terrain_geom.buffer();
        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
            ])
            .arrays(&[terrain_buf])
            .textures(&[
                &self.textures.tile_atlas,
                &cavern_tex,
            ])
            .range(0 .. terrain_buf.len() / mem::size_of::<graphics::terrain::Vertex>())
            .draw(&mut self.shaders.terrain);
    }

    pub fn render_structures(&mut self, scene: &Scene, cavern_tex: &GL::Texture, shadow: bool) {
        let structure_buf = self.structure_geom.buffer();
        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
                scene.now(),
            ])
            .arrays(&[structure_buf])
            .textures(&[
                &self.textures.structure_atlas,
                &cavern_tex,
            ])
            .range(0 .. structure_buf.len() / mem::size_of::<graphics::structure::Vertex>())
            .draw(if !shadow { &mut self.shaders.structure }
                  else { &mut self.shaders.structure_shadow });
    }

    pub fn render_static_lights(&mut self, scene: &Scene, depth_tex: &GL::Texture) {
        let light_buf = self.light_geom.buffer();
        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
            ])
            .arrays(&[light_buf])
            .textures(&[
                &depth_tex,
            ])
            .range(0 .. light_buf.len() / mem::size_of::<graphics::light::Vertex>())
            .draw(&mut self.shaders.light_static);
    }

    pub fn render_output(&mut self, tex: &GL::Texture) {
        DrawArgs::<GL>::new()
            .arrays(&[&self.buffers.square01])
            .textures(&[tex])
            .range(0..6)
            .draw(&mut self.shaders.blit_full);
    }

    pub fn render(&mut self, scene: &Scene, cavern_tex: &GL::Texture) {
        self.framebuffers.world.clear((0, 0, 0, 0));

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
            ])
            .arrays(&[self.terrain_geom.buffer()])
            .textures(&[
                &self.textures.tile_atlas,
                &cavern_tex,
            ])
            .output(&self.framebuffers.world)
            .draw(&mut self.shaders.terrain);

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
                scene.now(),
            ])
            .arrays(&[self.structure_geom.buffer()])
            .textures(&[
                &self.textures.structure_atlas,
                &cavern_tex,
            ])
            .output(&self.framebuffers.world)
            .draw(&mut self.shaders.structure);

        DrawArgs::<GL>::new()
            .arrays(&[&self.buffers.square01])
            .textures(&[self.framebuffers.world.color_texture(0)])
            .viewport_size(V2::new(1280, 960))
            .draw(&mut self.shaders.blit_full);
    }
}


pub struct Scene {
    camera_pos: [f32; 2],
    camera_size: [f32; 2],
    slice_center: [f32; 2],
    slice_z: f32,
    now: f32,
}

impl Scene {
    pub fn camera_pos(&self) -> UniformValue {
        UniformValue::V2(&self.camera_pos)
    }

    pub fn camera_size(&self) -> UniformValue {
        UniformValue::V2(&self.camera_size)
    }

    pub fn slice_center(&self) -> UniformValue {
        UniformValue::V2(&self.slice_center)
    }

    pub fn slice_z(&self) -> UniformValue {
        UniformValue::Float(self.slice_z)
    }

    pub fn now(&self) -> UniformValue {
        UniformValue::Float(self.now)
    }
}


struct GeomCache<GL: Context, K: Eq> {
    buffer: GL::Buffer,
    last_key: Option<K>,
}

impl<GL: Context, K: Eq> GeomCache<GL, K> {
    pub fn new(gl: &mut GL) -> GeomCache<GL, K> {
        GeomCache {
            buffer: gl.create_buffer(),
            last_key: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.last_key = None;
    }

    pub fn is_valid(&self, k: &K) -> bool {
        if let Some(ref last_key) = self.last_key {
            last_key == k
        } else {
            false
        }
    }

    pub fn buffer(&self) -> &GL::Buffer {
        &self.buffer
    }

    pub fn update<F>(&mut self, k: K, f: F)
            where F: FnOnce(&mut GL::Buffer, &K) {
        if !self.is_valid(&k) {
            f(&mut self.buffer, &k);
            self.last_key = Some(k);
        }
    }
}


fn load_buffer<GL, G>(buf: &mut GL::Buffer,
                      gen: &mut G,
                      tmp: &mut [G::Vertex],
                      offset: &mut usize)
        where GL: Context, G: GeometryGenerator {
    let mut keep_going = true;
    while keep_going {
        let (len, more) = gen.generate(tmp);
        keep_going = more;

        let byte_len = len * mem::size_of::<G::Vertex>();
        let bytes = unsafe {
            slice::from_raw_parts(tmp.as_ptr() as *const u8, byte_len)
        };
        buf.load(*offset, bytes);
        *offset += byte_len;
    }
}


