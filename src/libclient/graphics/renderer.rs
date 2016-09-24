use std::prelude::v1::*;
use types::*;
use std::mem;
use std::slice;
use physics::floodfill;
use physics::TILE_SIZE;
use physics::v3::{V3, V2, Vn, scalar, Region};

use data::Data;
use debug;
use entity::Entities;
use graphics::GeometryGenerator;
use graphics::types::LocalChunks;
use platform::gl::{Context, Buffer, Framebuffer, Texture};
use platform::gl::{DrawArgs, UniformValue, Attach, BlendMode, DepthMode, Feature, FeatureStatus};
use structures::Structures;
use terrain::{LOCAL_SIZE, LOCAL_MASK};
use ui;
use util;

use super::entity;
use super::light;
use super::structure;
use super::terrain;


pub const RAW_MODULUS: i32 = 55440;

// The `now` value passed to the animation shader must be reduced to fit in a
// float.  We use the magic number 55440 for this, since it's divisible by
// every number from 1 to 12 (and most "reasonable" numbers above that).  This
// is useful because repeating animations will glitch when `now` wraps around
// unless `length / framerate` divides evenly into the modulus.
//
// Note that the shader `now` and ANIM_MODULUS are both in seconds, not ms.
pub const ANIM_MODULUS: f64 = RAW_MODULUS as f64;

// We also need a smaller modulus for one-shot animation start times.  These
// are measured in milliseconds and must fit in a 16-bit int.  It's important
// that the one-shot modulus divides evenly into 1000 * ANIM_MODULUS, because
// the current frame time in milliseconds will be modded by 1000 * ANIM_MODULUS
// and then again by the one-shot modulus.
//
// We re-use ANIM_MODULUS as the one-shot modulus, since it obviously divides
// evenly into 1000 * ANIM_MODULUS.  This is okay as long as ANIM_MODULUS fits
// into 16 bits.
pub const ONESHOT_MODULUS: Time = ANIM_MODULUS as Time;


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
    blit_post: GL::Shader,
    ui: GL::Shader,

    terrain: GL::Shader,
    structure: GL::Shader,
    structure_shadow: GL::Shader,
    light_static: GL::Shader,
    entity: GL::Shader,

    cursor: GL::Shader,
    debug_graph: GL::Shader,
}

impl<GL: Context> Shaders<GL> {
    fn new(gl: &mut GL) -> Shaders<GL> {
        Shaders {
            blit_full: gl.load_shader(
                "blit_fullscreen.vert", "blit_output.frag", "",
                uniforms! {},
                arrays! {
                    [2] attribs! {
                        corner: U8[2] @0,
                    },
                },
                textures! { image_tex, },
                outputs! { color: 1 }),

            blit_post: gl.load_shader(
                "blit_fullscreen.vert", "blit_post.frag", "",
                uniforms! {
                    screen_size: V2,
                },
                arrays! {
                    [2] attribs! {
                        corner: U8[2] @0,
                    },
                },
                textures! {
                    color_tex,
                    meta_tex,
                    depth_tex,
                    entity_depth_tex,
                    light_tex,
                },
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

            cursor: gl.load_shader(
                "cursor.vert", "cursor.frag", "",
                uniforms! {
                    camera_pos: V2,
                    camera_size: V2,
                    cursor_pos: V2,
                },
                arrays! {
                    [2] attribs! {
                        corner: U8[2] @0,
                    },
                },
                textures! {},
                outputs! { color: 1 }),

            debug_graph: gl.load_shader(
                "debug_graph.vert", "debug_graph.frag", "",
                uniforms! {
                    screen_size: V2,
                    graph_pos: V2,
                    graph_size: V2,
                    cur_index: Float,
                },
                arrays! {
                    [2] attribs! {
                        corner: U8[2] @0,
                    },
                },
                textures! { data_tex, },
                outputs! { color: 1 }),
        }
    }
}

struct Textures<GL: Context> {
    tile_atlas: GL::Texture,
    structure_atlas: GL::Texture,
    sprite_sheet: GL::Texture,

    cavern_map: GL::Texture,
    debug_graph_data: GL::Texture,

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
            debug_graph_data: gl.create_texture((debug::NUM_FRAMES as u16, 1)),

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
    world_entity_depth: GL::Texture,
    light_color: GL::Texture,
    shadow_color: GL::Texture,
    output_color: GL::Texture,

    world: GL::Framebuffer,
    world_and_meta: GL::Framebuffer,
    light: GL::Framebuffer,
    shadow: GL::Framebuffer,
    sprite: GL::Framebuffer,
    output: GL::Framebuffer,
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
        let world_entity_depth = gl.create_depth_texture(size);
        let light_color = gl.create_texture(size);
        let shadow_color = gl.create_texture(size);
        let output_color = gl.create_texture(size);

        let world = gl.create_framebuffer(size,
                                          &[Attach::Texture(&world_color)],
                                          Some(Attach::Texture(&world_depth)));
        let world_and_meta = gl.create_framebuffer(size,
                                                   &[Attach::Texture(&world_color),
                                                     Attach::Texture(&world_meta) ],
                                                   Some(Attach::Texture(&world_depth)));
        let light = gl.create_framebuffer(size,
                                          &[Attach::Texture(&light_color)],
                                          None);
        let shadow = gl.create_framebuffer(size,
                                           &[Attach::Texture(&shadow_color)],
                                           Some(Attach::Texture(&world_depth)));
        let sprite = gl.create_framebuffer(size,
                                           &[Attach::Texture(&world_color)],
                                           Some(Attach::Texture(&world_entity_depth)));
        let output = gl.create_framebuffer(size,
                                           &[Attach::Texture(&output_color)],
                                           None);

        Framebuffers {
            size: size,

            world_color: world_color,
            world_meta: world_meta,
            world_depth: world_depth,
            world_entity_depth: world_entity_depth,
            light_color: light_color,
            shadow_color: shadow_color,
            output_color: output_color,

            world: world,
            world_and_meta: world_and_meta,
            light: light,
            shadow: shadow,
            sprite: sprite,
            output: output,
        }
    }
}

pub struct Renderer<GL: Context> {
    terrain_geom: GeomCache<GL, Region<V2>>,
    structure_geom: GeomCache<GL, Region<V2>>,
    structure_light_geom: GeomCache<GL, Region<V2>>,
    entity_buffer: GL::Buffer,
    entity_light_buffer: GL::Buffer,
    ui_buffer: GL::Buffer,
    ui_special: Vec<ui::geom::Special>,

    buffers: Buffers<GL>,
    shaders: Shaders<GL>,
    textures: Textures<GL>,
    framebuffers: Framebuffers<GL>,

    pub render_names: bool,
}

impl<GL: Context> Renderer<GL> {
    pub fn new(gl: &mut GL) -> Renderer<GL> {
        Renderer {
            terrain_geom: GeomCache::new(gl),
            structure_geom: GeomCache::new(gl),
            structure_light_geom: GeomCache::new(gl),
            entity_buffer: gl.create_buffer(),
            entity_light_buffer: gl.create_buffer(),
            ui_buffer: gl.create_buffer(),
            ui_special: Vec::new(),

            buffers: Buffers::new(gl),
            shaders: Shaders::new(gl),
            textures: Textures::new(gl),
            framebuffers: Framebuffers::new(gl, (640, 480)),

            render_names: true,
        }
    }


    pub fn update_terrain_geometry(&mut self,
                                   data: &Data,
                                   chunks: &LocalChunks,
                                   bounds: Region<V2>) {
        self.terrain_geom.update(bounds, |buffer, _| {
            let mut gen = terrain::RegionGeomGen::new(data.blocks(), chunks, bounds);
            load_buffer::<GL, _>(buffer, &mut gen);
        });
    }

    pub fn invalidate_terrain_geometry(&mut self) {
        self.terrain_geom.invalidate();
    }


    pub fn update_structure_geometry(&mut self,
                                     data: &Data,
                                     structures: &Structures,
                                     bounds: Region<V2>) {
        self.structure_geom.update(bounds, |buffer, _| {
            let mut gen = structure::GeomGen::new(structures, data, bounds);
            load_buffer::<GL, _>(buffer, &mut gen);
        });
    }

    pub fn invalidate_structure_geometry(&mut self) {
        self.structure_geom.invalidate();
        self.structure_light_geom.invalidate();
    }


    pub fn update_structure_light_geometry(&mut self,
                                           data: &Data,
                                           structures: &Structures,
                                           bounds: Region<V2>) {
        self.structure_light_geom.update(bounds, |buffer, _| {
            let mut gen = light::StructureGeomGen::new(structures, data.templates(), bounds);
            load_buffer::<GL, _>(buffer, &mut gen);
        });
    }

    pub fn invalidate_structure_light_geometry(&mut self) {
        self.structure_light_geom.invalidate();
    }


    pub fn load_ui_geometry(&mut self, geom: &[ui::geom::Vertex]) {
        let byte_len = geom.len() * mem::size_of::<ui::geom::Vertex>();
        let bytes = unsafe {
            slice::from_raw_parts(geom.as_ptr() as *const u8, byte_len)
        };

        self.ui_buffer.alloc(byte_len);
        self.ui_buffer.load(0, bytes);
    }

    pub fn set_ui_special(&mut self, special: Vec<ui::geom::Special>) {
        self.ui_special = special;
    }


    pub fn update_entity_geometry(&mut self,
                                  data: &Data,
                                  entities: &Entities,
                                  bounds: Region<V2>,
                                  now: Time) {
        {
            let mut gen = entity::GeomGen::new(entities, data, self.render_names,
                                               bounds, now);
            load_buffer::<GL, _>(&mut self.entity_buffer, &mut gen);
        }

        {
            let mut gen = light::EntityGeomGen::new(entities, bounds, now);
            load_buffer::<GL, _>(&mut self.entity_light_buffer, &mut gen);
        }
    }


    pub fn load_cavern_map(&mut self, data: &[floodfill::flags::Flags]) {
        assert!(mem::size_of::<floodfill::flags::Flags>() == mem::size_of::<u8>());
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
        self.framebuffers.light.clear(scene.ambient_light);
        self.framebuffers.shadow.clear((0, 0, 0, 0));
        self.framebuffers.sprite.clear((0, 0, 0, 0));

        let mut anim_now = scene.now as f64 / 1000.0 % ANIM_MODULUS;
        if anim_now < 0.0 {
            anim_now += ANIM_MODULUS;
        }
        let anim_now = anim_now;

        self.render_terrain(scene);
        self.render_structures(scene, anim_now);
        self.render_structure_shadows(scene, anim_now);
        self.render_sprites(scene, anim_now);
        self.render_lights(scene);
        self.render_postprocess(scene);
        self.render_ui(scene);

        DrawArgs::<GL>::new()
            .arrays(&[&self.buffers.square01])
            .textures(&[&self.framebuffers.output_color])
            .viewport_size(scene.canvas_size)
            .draw(&mut self.shaders.blit_full);
    }

    pub fn render_ponyedit_hack(&mut self, scene: &Scene) {
        self.framebuffers.world.clear((0, 0, 0, 0));
        self.framebuffers.sprite.clear((0, 0, 0, 0));

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
                UniformValue::Float(0.0),
            ])
            .arrays(&[&self.entity_buffer])
            .textures(&[
                &self.textures.sprite_sheet,
                &self.framebuffers.world_depth,
                &self.textures.cavern_map,
            ])
            .output(&self.framebuffers.sprite)
            .viewport_size(scene.camera_size)
            .draw(&mut self.shaders.entity);

        DrawArgs::<GL>::new()
            .arrays(&[&self.buffers.square01])
            .textures(&[&self.framebuffers.world_color])
            .viewport_size(scene.canvas_size)
            .draw(&mut self.shaders.blit_full);
    }

    fn render_terrain(&mut self, scene: &Scene) {
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
            .output(&self.framebuffers.world_and_meta)
            .depth_test(DepthMode::GEqual)
            .draw(&mut self.shaders.terrain);
    }

    fn render_structures(&mut self, scene: &Scene, anim_now: f64) {
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
            .output(&self.framebuffers.world_and_meta)
            .depth_test(DepthMode::GEqual)
            .draw(&mut self.shaders.structure);
    }

    fn render_structure_shadows(&mut self, scene: &Scene, anim_now: f64) {
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
            .output(&self.framebuffers.shadow)
            .depth_test(DepthMode::GEqual)
            .draw(&mut self.shaders.structure_shadow);

        // Apply shadows
        DrawArgs::<GL>::new()
            .arrays(&[&self.buffers.square01])
            .textures(&[&self.framebuffers.shadow_color])
            .output(&self.framebuffers.world)
            .blend_mode(BlendMode::Alpha)
            .draw(&mut self.shaders.blit_full);
    }

    fn render_sprites(&mut self, scene: &Scene, anim_now: f64) {
        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
                scene.slice_center(),
                scene.slice_z(),
                UniformValue::Float(anim_now as f32),
            ])
            .arrays(&[&self.entity_buffer])
            .textures(&[
                &self.textures.sprite_sheet,
                &self.framebuffers.world_depth,
                &self.textures.cavern_map,
            ])
            .output(&self.framebuffers.sprite)
            .depth_test(DepthMode::Always)
            .draw(&mut self.shaders.entity);
    }

    fn render_lights(&mut self, scene: &Scene) {
        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
            ])
            .arrays(&[self.structure_light_geom.buffer()])
            .textures(&[
                &self.framebuffers.world_depth,
                &self.framebuffers.world_entity_depth,
            ])
            .output(&self.framebuffers.light)
            .blend_mode(BlendMode::MultiplyInv)
            .draw(&mut self.shaders.light_static);

        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.camera_pos(),
                scene.camera_size(),
            ])
            .arrays(&[&self.entity_light_buffer])
            .textures(&[
                &self.framebuffers.world_depth,
                &self.framebuffers.world_entity_depth,
            ])
            .output(&self.framebuffers.light)
            .blend_mode(BlendMode::MultiplyInv)
            .draw(&mut self.shaders.light_static);
    }

    fn render_ui(&mut self, scene: &Scene) {
        // Main UI rendering
        DrawArgs::<GL>::new()
            .uniforms(&[
                scene.ui_camera_size(),
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
            .output(&self.framebuffers.output)
            .blend_mode(BlendMode::Alpha)
            .draw(&mut self.shaders.ui);

        // Handle special parts
        for s in &self.ui_special {
            use ui::geom::Special::*;
            match *s {
                DebugFrameGraph { rect, cur, last, last_time, last_interval } => {
                    let bytes = [(last_time & 0xff) as u8,
                                 (last_time >> 8) as u8,
                                 (last_interval & 0xff) as u8,
                                 (last_interval >> 8) as u8];
                    let tex_rect = Region::<V2>::sized(scalar(1)) + V2::new(last as i32, 0);
                    self.textures.debug_graph_data.load_partial(tex_rect, &bytes);

                    DrawArgs::<GL>::new()
                        .uniforms(&[
                            scene.ui_camera_size(),
                            UniformValue::V2(&v2_float(rect.min)),
                            UniformValue::V2(&v2_float(rect.size())),
                            UniformValue::Float(cur as f32),
                        ])
                        .arrays(&[&self.buffers.square01])
                        .textures(&[&self.textures.debug_graph_data])
                        .output(&self.framebuffers.output)
                        .draw(&mut self.shaders.debug_graph);
                }
            }
        }
    }

    fn render_postprocess(&mut self, scene: &Scene) {
        DrawArgs::<GL>::new()
            .uniforms(&[scene.camera_size()])
            .arrays(&[&self.buffers.square01])
            .textures(&[
                      &self.framebuffers.world_color,
                      &self.framebuffers.world_meta,
                      &self.framebuffers.world_depth,
                      &self.framebuffers.world_entity_depth,
                      &self.framebuffers.light_color,
            ])
            .output(&self.framebuffers.output)
            .draw(&mut self.shaders.blit_post);

        if scene.cursor_pos.is_some() {
            println!("draw cursor at {:?}", scene.cursor_pos);
            DrawArgs::<GL>::new()
                .uniforms(&[
                      scene.camera_pos(),
                      scene.camera_size(),
                      scene.cursor_pos(),
                ])
                .arrays(&[&self.buffers.square01])
                .output(&self.framebuffers.output)
                .draw(&mut self.shaders.cursor);
        }
    }
}

fn v2_float(v: V2) -> [f32; 2] {
    [v.x as f32, v.y as f32]
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
    pub ambient_light: (u8, u8, u8, u8),
    pub cursor_pos: Option<V2>,

    pub f_camera_pos: [f32; 2],
    pub f_camera_size: [f32; 2],
    pub f_ui_camera_size: [f32; 2],
    pub f_slice_center: [f32; 2],
    pub f_slice_z: f32,
    pub f_cursor_pos: [f32; 2],
}

impl Scene {
    pub fn new(now: Time,
               window_size: (u16, u16),
               view_size: (u16, u16),
               ui_scale: u16,
               center: V3,
               ambient_light: (u8, u8, u8, u8),
               cursor_pos: Option<V2>) -> Scene {
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
            ambient_light: ambient_light,
            cursor_pos: cursor_pos,

            f_camera_pos: [camera_pos.x as f32,
                           camera_pos.y as f32],
            f_camera_size: [camera_size.x as f32,
                            camera_size.y as f32],
            f_ui_camera_size: [camera_size.x as f32 / ui_scale as f32,
                               camera_size.y as f32 / ui_scale as f32],
            f_slice_center: [slice_center.x as f32,
                             slice_center.y as f32],
            f_slice_z: slice_center.z as f32,
            f_cursor_pos:
                if let Some(pos) = cursor_pos {
                    [pos.x as f32, pos.y as f32]
                } else {
                    [0.0, 0.0]
                },
        }
    }

    fn camera_pos(&self) -> UniformValue {
        UniformValue::V2(&self.f_camera_pos)
    }

    fn camera_size(&self) -> UniformValue {
        UniformValue::V2(&self.f_camera_size)
    }

    fn ui_camera_size(&self) -> UniformValue {
        UniformValue::V2(&self.f_ui_camera_size)
    }

    fn slice_center(&self) -> UniformValue {
        UniformValue::V2(&self.f_slice_center)
    }

    fn slice_z(&self) -> UniformValue {
        UniformValue::Float(self.f_slice_z)
    }

    fn cursor_pos(&self) -> UniformValue {
        UniformValue::V2(&self.f_cursor_pos)
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
                      gen: &mut G)
        where GL: Context, G: GeometryGenerator {
    buf.alloc(gen.count_verts() * mem::size_of::<G::Vertex>());

    let mut tmp = unsafe { util::zeroed_boxed_slice(64 * 1024) };
    let mut idx = 0;

    let mut buf_idx = 0;
    let mut flush = |tmp: &[G::Vertex], count| {
        if count == 0 {
            return;
        }

        let byte_len = count * mem::size_of::<G::Vertex>();
        let bytes = unsafe {
            slice::from_raw_parts(tmp.as_ptr() as *const u8, byte_len)
        };
        buf.load(buf_idx, bytes);
        buf_idx += bytes.len();
    };

    gen.generate(|v| {
        if idx == tmp.len() {
            flush(&tmp, idx);
            idx = 0;
        }
        tmp[idx] = v;
        idx += 1;
    });
    flush(&tmp, idx);
}
