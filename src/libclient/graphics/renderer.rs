use std::prelude::v1::*;
use std::mem;
use std::slice;

use physics::fill_flags;
use physics::TILE_SIZE;
use physics::v3::{V3, V2, Vn, scalar, Region};

use Time;
use data::Data;
use entity::{Entities, EntityId};
use graphics;
use graphics::GeometryGenerator;
use graphics::types::LocalChunks;
use platform::gl::{Context, Buffer, Framebuffer, Texture};
use platform::gl::{DrawArgs, UniformValue, Attach, BlendMode, Feature, FeatureStatus};
use predict::Predictor;
use structures::Structures;
use terrain::{LOCAL_SIZE, LOCAL_MASK};
use ui;
use util;

use super::entity;
use super::light;
use super::structure;
use super::terrain;


// The `now` value passed to the animation shader must be reduced to fit in a
// float.  We use the magic number 55440 for this, since it's divisible by
// every number from 1 to 12 (and most "reasonable" numbers above that).  This
// is useful because repeating animations will glitch when `now` wraps around
// unless `length / framerate` divides evenly into the modulus.
//
// Note that the shader `now` and ANIM_MODULUS are both in seconds, not ms.
const ANIM_MODULUS: f64 = 55440.0;

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
    ui: GL::Shader,

    terrain: GL::Shader,
    structure: GL::Shader,
    structure_shadow: GL::Shader,
    light_static: GL::Shader,
    entity: GL::Shader,
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

            ui: gl.load_shader(
                "ui_blit2.vert", "ui_blit2.frag", "",
                uniforms! {
                    screenSize: V2,
                    sheetSize0: V2,
                    sheetSize1: V2,
                    sheetSize2: V2,
                },
                arrays! {
                    [16] attribs! {
                        srcPos: U16[2] @0,
                        srcSize: U8[2] @4,
                        sheetAttr: U8[1] @6,
                        dest: I16[2] @8,
                        offset_: U16[2] @12,
                    },
                },
                textures! { sheet0, sheet1, sheet2, },
                outputs! { color: 1 }),

            terrain: terrain::load_shader(gl),

            structure: structure::load_shader(gl, false),
            structure_shadow: structure::load_shader(gl, true),

            light_static: light::load_shader(gl),

            entity: entity::load_shader(gl),
        }
    }
}

struct Textures<GL: Context> {
    tile_atlas: GL::Texture,
    structure_atlas: GL::Texture,
    sprite_sheet: GL::Texture,

    cavern_map: GL::Texture,

    ui_items: GL::Texture,
    ui_parts: GL::Texture,
    ui_fonts: GL::Texture,
}

impl<GL: Context> Textures<GL> {
    fn new(gl: &mut GL) -> Textures<GL> {
        Textures {
            tile_atlas: gl.load_texture("tiles"),
            structure_atlas: gl.load_texture("structures0"),
            sprite_sheet: gl.load_texture("sprites0"),

            cavern_map: gl.create_luminance_texture((96, 96)),

            ui_items: gl.load_texture("items_img"),
            ui_parts: gl.load_texture("ui_atlas"),
            ui_fonts: gl.load_texture("fonts"),
        }
    }
}

struct Framebuffers<GL: Context> {
    size: (u16, u16),

    world_color: GL::Texture,
    world_meta: GL::Texture,
    world_depth: GL::Texture,

    world: GL::Framebuffer,
    sprite: GL::Framebuffer,
}

fn feature_check_one<GL: Context>(gl: &GL, feature: Feature) -> bool {
    match gl.check_feature(feature) {
        FeatureStatus::Unavailable => {
            error!("required OpenGL feature is unavailable: {:?}", feature);
            false
        },
        FeatureStatus::Emulated => {
            warn!("OpenGL feature is being emulated (may be slow): {:?}", feature);
            true
        },
        FeatureStatus::Native => {
            true
        },
    }
}

fn feature_check<GL: Context>(gl: &GL) {
    assert!(
        // NB: use & instead of && so that all missing features are reported at once
        feature_check_one(gl, Feature::DepthTexture) &
        feature_check_one(gl, Feature::MultiPlaneFramebuffer) &
        true,
        "some required OpenGL features are unavailable");
}

impl<GL: Context> Framebuffers<GL> {
    fn new(gl: &mut GL, size: (u16, u16)) -> Framebuffers<GL> {
        feature_check(gl);

        let world_color = gl.create_texture(size);
        let world_meta = gl.create_texture(size);
        let world_depth = gl.create_depth_texture(size);
        let world = gl.create_framebuffer(size,
                                          &[Attach::Texture(&world_color),
                                            Attach::Texture(&world_meta) ],
                                          Some(Attach::Texture(&world_depth)));
        let sprite = gl.create_framebuffer(size,
                                           &[Attach::Texture(&world_color)],
                                           Some(Attach::Renderbuffer));

        Framebuffers {
            size: size,

            world_color: world_color,
            world_meta: world_meta,
            world_depth: world_depth,

            world: world,
            sprite: sprite,
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
                                  predictor: &Predictor,
                                  bounds: Region<V2>,
                                  now: Time,
                                  future: Time,
                                  pawn_id: Option<EntityId>) {
        let mut gen = entity::GeomGen::new(entities, predictor, data,
                                           bounds, now, future, pawn_id);
        self.entity_buffer.alloc(gen.count_verts() * mem::size_of::<entity::Vertex>());
        let mut tmp = unsafe { util::zeroed_boxed_slice(64 * 1024) };
        load_buffer::<GL, _>(&mut self.entity_buffer, &mut gen, &mut tmp, &mut 0);
    }


    pub fn load_cavern_map(&mut self, data: &[fill_flags::Flags]) {
        assert!(mem::size_of::<fill_flags::Flags>() == mem::size_of::<u8>());
        let raw_data: &[u8] = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8,
                                  data.len())
        };
        self.textures.cavern_map.load(raw_data);
    }


    pub fn update_framebuffers(&mut self, gl: &mut GL, scene: &Scene) {
        let u16_size = (scene.camera_size.x as u16,
                        scene.camera_size.y as u16);
        if self.framebuffers.size != u16_size {
            println!("adjusting framebuffer size to {:?}", scene.camera_size);
            self.framebuffers = Framebuffers::new(gl, u16_size);
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        self.framebuffers.world.clear((0, 0, 0, 0));
        self.framebuffers.sprite.clear((0, 0, 0, 0));

        let mut anim_now = scene.now as f64 / 1000.0 % ANIM_MODULUS;
        if anim_now < 0.0 {
            anim_now += ANIM_MODULUS;
        }
        let anim_now = anim_now;

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
                &self.textures.cavern_map,
            ])
            .output(&self.framebuffers.world)
            .depth_test()
            .draw(&mut self.shaders.terrain);

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
                UniformValue::Float(anim_now as f32),
            ])
            .arrays(&[self.structure_geom.buffer()])
            .textures(&[
                &self.textures.structure_atlas,
                &self.textures.cavern_map,
            ])
            .output(&self.framebuffers.world)
            .depth_test()
            .draw(&mut self.shaders.structure);

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                //scene.slice_center(),
                //scene.slice_z(),
                UniformValue::Float(anim_now as f32),
            ])
            .arrays(&[&self.entity_buffer])
            .textures(&[
                &self.textures.sprite_sheet,
                &self.framebuffers.world_depth,
            ])
            .output(&self.framebuffers.sprite)
            .depth_test()
            .draw(&mut self.shaders.entity);

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_size(),
                UniformValue::V2(&size_float(&self.textures.ui_items)),
                UniformValue::V2(&size_float(&self.textures.ui_parts)),
                UniformValue::V2(&size_float(&self.textures.ui_fonts)),
            ])
            .arrays(&[&self.ui_buffer])
            .textures(&[
                &self.textures.ui_items,
                &self.textures.ui_parts,
                &self.textures.ui_fonts,
            ])
            .output(&self.framebuffers.world)
            .blend_mode(BlendMode::Alpha)
            .draw(&mut self.shaders.ui);

        DrawArgs::<GL>::new()
            .arrays(&[&self.buffers.square01])
            .textures(&[&self.framebuffers.world_color])
            .viewport_size(scene.canvas_size)
            .draw(&mut self.shaders.blit_full);
    }
}

fn size_float<T: Texture>(t: &T) -> [f32; 2] {
    let (w,h) = t.size();
    [w as f32, h as f32]
}


pub struct Scene {
    pub canvas_size: V2,
    pub camera_pos: V2,
    pub camera_size: V2,
    pub slice_center: V3,
    pub now: Time,

    pub f_camera_pos: [f32; 2],
    pub f_camera_size: [f32; 2],
    pub f_slice_center: [f32; 2],
    pub f_slice_z: f32,
}

impl Scene {
    pub fn new(now: Time,
               window_size: (u16, u16),
               view_size: (u16, u16),
               center: V3) -> Scene {
        let camera_center = V2::new(center.x, center.y - center.z);
        let camera_size = V2::new(view_size.0 as i32, view_size.1 as i32);
        let canvas_size = V2::new(window_size.0 as i32, window_size.1 as i32);
        let camera_pos = camera_center - camera_size / scalar(2);

        let slice_center = center.div_floor(scalar(TILE_SIZE));

        Scene {
            canvas_size: canvas_size,
            camera_pos: camera_pos,
            camera_size: camera_size,
            slice_center: slice_center,
            now: now,

            f_camera_pos: [camera_pos.x as f32,
                           camera_pos.y as f32],
            f_camera_size: [camera_size.x as f32,
                            camera_size.y as f32],
            f_slice_center: [slice_center.x as f32,
                             slice_center.y as f32],
            f_slice_z: slice_center.z as f32,
        }
    }

    fn camera_pos(&self) -> UniformValue {
        UniformValue::V2(&self.f_camera_pos)
    }

    fn camera_size(&self) -> UniformValue {
        UniformValue::V2(&self.f_camera_size)
    }

    fn slice_center(&self) -> UniformValue {
        UniformValue::V2(&self.f_slice_center)
    }

    fn slice_z(&self) -> UniformValue {
        UniformValue::Float(self.f_slice_z)
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


