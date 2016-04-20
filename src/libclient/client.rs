use std::prelude::v1::*;
use std::boxed::FnBox;
use std::cmp;
use std::mem;

use platform::{Platform, PlatformObj};
use platform::{Config, ConfigKey};
use platform::Cursor;
use platform::gl::Context as GlContext;
use util;

use graphics::types::{BlockChunk, LocalChunks};
use graphics::types::HAS_LIGHT;
use graphics::renderer::Renderer;

use physics;
use physics::{CHUNK_SIZE, CHUNK_BITS, TILE_SIZE};
use physics::Shape;
use physics::v3::{V3, V2, scalar, Region};

use Time;
use data::Data;
use entity::{Entities, EntityId, Motion};
use graphics::renderer::Scene;
use graphics::types::StructureTemplate;
use inventory::{Inventories, Item, InventoryId};
use structures::Structures;
use terrain::TerrainShape;
use terrain::{LOCAL_SIZE, LOCAL_BITS};
use ui::{UI, Dyn};
use ui::input::{KeyAction, EventStatus};


pub struct Client<'d, P: Platform> {
    data: &'d Data,
    platform: P,

    chunks: Box<LocalChunks>,
    terrain_shape: Box<TerrainShape>,
    structures: Structures,
    entities: Entities,
    inventories: Inventories,

    ui: UI,

    renderer: Renderer<P::GL>,

    pawn_id: Option<EntityId>,
    window_size: (u16, u16),
    view_size: (u16, u16),

    last_cursor: Cursor,
}

impl<'d, P: Platform> Client<'d, P> {
    pub fn new(data: &'d Data, platform: P) -> Client<'d, P> {
        let mut platform = platform;
        let renderer = Renderer::new(platform.gl());

        let mut c = Client {
            data: data,
            platform: platform,

            chunks: box [[0; 1 << (3 * CHUNK_BITS)]; 1 << (2 * LOCAL_BITS)],
            terrain_shape: box TerrainShape::new(),
            structures: Structures::new(),
            entities: Entities::new(),
            inventories: Inventories::new(),

            ui: UI::new(),

            renderer: renderer,

            pawn_id: None,
            window_size: (640, 480),
            view_size: (640, 480),

            last_cursor: Cursor::Normal,
        };

        c.ui.root.init_hotbar(c.platform.config(), &c.data);

        c
    }


    pub fn reset_all(&mut self) {
        for chunk in self.chunks.iter_mut() {
            for block in chunk.iter_mut() {
                *block = 0;
            }
        }

        self.terrain_shape.clear();
        self.structures.clear();
        self.entities.clear();
        self.inventories.clear();

        self.pawn_id = None;

        // TODO: close any open dialog

        self.renderer.invalidate_terrain_geometry();
        self.renderer.invalidate_structure_geometry();
        self.renderer.invalidate_light_geometry();
    }

    // Terrain chunk tracking

    pub fn load_terrain_chunk(&mut self, cpos: V2, data: &[u16]) {
        // Update self.chunks
        let bounds = Region::new(scalar(0), scalar(LOCAL_SIZE));
        let blocks = &mut self.chunks[bounds.index(cpos)];
        rle16Decode(data, blocks);

        // Update self.terrain_shape
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

    // Structure tracking

    pub fn add_structure_shape(&mut self,
                               t: &StructureTemplate,
                               pos: (u8, u8, u8)) {
        let pos = util::unpack_v3(pos);
        let size = util::unpack_v3(t.size);
        let bounds = Region::new(pos, pos + size);
        let base = t.shape_idx as usize;
        let shape = &self.data.template_shapes[base .. base + bounds.volume() as usize];
        self.terrain_shape.set_shape_in_region(bounds, 1 + t.layer as usize, shape);
    }

    pub fn remove_structure_shape(&mut self,
                                  t: &StructureTemplate,
                                  pos: (u8, u8, u8)) {
        let pos = util::unpack_v3(pos);
        let size = util::unpack_v3(t.size);
        let bounds = Region::new(pos, pos + size);
        self.terrain_shape.fill_shape_in_region(bounds, 1 + t.layer as usize, Shape::Empty);
    }

    pub fn structure_appear(&mut self,
                            id: u32,
                            pos: (u8, u8, u8),
                            template_id: u32,
                            oneshot_start: u16) {
        // Update self.structures
        self.structures.insert(id, pos, template_id, oneshot_start);

        // Update self.terrain_cache
        let t = &self.data.templates[template_id as usize];
        self.add_structure_shape(t, pos);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
        if t.flags.contains(HAS_LIGHT) {
            self.renderer.invalidate_light_geometry();
        }
    }

    pub fn structure_gone(&mut self,
                          id: u32) {
        // Update self.structures
        let s = self.structures.remove(id);

        // Update self.terrain_cache
        let t = &self.data.templates[s.template_id as usize];
        self.remove_structure_shape(t, s.pos);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
        if t.flags.contains(HAS_LIGHT) {
            self.renderer.invalidate_light_geometry();
        }
    }

    pub fn structure_replace(&mut self,
                             id: u32,
                             template_id: u32,
                             oneshot_start: u16) {
        let (pos, old_t) = {
            let s = &self.structures[id];
            (s.pos,
             &self.data.templates[template_id as usize])
        };
        let new_t = &self.data.templates[template_id as usize];

        // Update self.structures
        self.structures.replace(id, template_id, oneshot_start);

        // Update self.terrain_cache
        self.remove_structure_shape(old_t, pos);
        self.add_structure_shape(new_t, pos);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
        if old_t.flags.contains(HAS_LIGHT) || new_t.flags.contains(HAS_LIGHT) {
            self.renderer.invalidate_light_geometry();
        }
    }

    // Entity tracking

    pub fn entity_appear(&mut self,
                         id: EntityId,
                         appearance: u32,
                         name: Option<String>) {
        self.entities.insert(id, appearance, name);
    }

    pub fn entity_gone(&mut self,
                       id: EntityId) {
        self.entities.remove(id);
    }

    pub fn entity_update(&mut self,
                         id: EntityId,
                         when: Time,
                         motion: Motion) {
        self.entities.schedule_update(id, when, motion);
    }

    pub fn set_pawn_id(&mut self,
                       id: EntityId) {
        self.pawn_id = Some(id);
    }

    pub fn clear_pawn_id(&mut self) {
        self.pawn_id = None;
    }

    // Inventory tracking

    pub fn inventory_appear(&mut self,
                            id: InventoryId,
                            items: Box<[Item]>) {
        self.inventories.insert(id, items);
    }

    pub fn inventory_gone(&mut self,
                          id: InventoryId) {
        self.inventories.remove(id);
    }

    pub fn inventory_update(&mut self,
                            id: InventoryId,
                            slot: usize,
                            item: Item) {
        self.inventories.update(id, slot, item);
    }

    pub fn set_main_inventory_id(&mut self, id: InventoryId) {
        self.inventories.set_main_id(id);
    }

    pub fn set_ability_inventory_id(&mut self, id: InventoryId) {
        self.inventories.set_ability_id(id);
    }

    // UI input

    fn process_event_status(&mut self, status: EventStatus) -> bool {
        match status {
            EventStatus::Unhandled => false,
            EventStatus::Handled => true,
            EventStatus::Action(f) => {
                // TODO: figure out why f(...) doesn't work
                f.call_box((self,));
                true
            },
        }
    }

    fn with_ui_dyn<F: FnOnce(&mut UI, Dyn) -> R, R>(&mut self, f: F) -> R {
        let dyn = Dyn::new(self.view_size,
                           &self.inventories);
        f(&mut self.ui, dyn)
    }

    pub fn input_key(&mut self, code: u8) -> bool {
        let status =
            if let Some(key) = KeyAction::from_code(code) {
                self.with_ui_dyn(|ui, dyn| ui.handle_key(key, dyn))
            } else {
                EventStatus::Unhandled
            };
        self.process_event_status(status)
    }

    pub fn input_mouse_move(&mut self, pos: V2) -> bool {
        let status = self.with_ui_dyn(|ui, dyn| ui.handle_mouse_move(pos, dyn));
        self.process_event_status(status)
    }

    pub fn input_mouse_down(&mut self, pos: V2) -> bool {
        let status = self.with_ui_dyn(|ui, dyn| ui.handle_mouse_down(pos, dyn));
        self.process_event_status(status)
    }

    pub fn input_mouse_up(&mut self, pos: V2) -> bool {
        let status = self.with_ui_dyn(|ui, dyn| ui.handle_mouse_up(pos, dyn));
        self.process_event_status(status)
    }

    pub fn open_inventory_dialog(&mut self) {
        use ui::dialogs::AnyDialog;
        self.ui.root.dialog.inner = AnyDialog::inventory();
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

    fn prepare(&mut self, scene: &Scene) {
        let bounds = Region::sized(scene.camera_size) + scene.camera_pos;
        let chunk_bounds = bounds.div_round_signed(CHUNK_SIZE * TILE_SIZE);

        // Terrain from the chunk below can cover the current one.
        let terrain_bounds = Region::new(chunk_bounds.min - V2::new(0, 0),
                                         chunk_bounds.max + V2::new(0, 1));
        self.renderer.update_terrain_geometry(&self.data, &self.chunks, terrain_bounds);

        // Structures from the chunk below can cover the current one, and also
        // structures from chunks above and to the left can extend into it.
        let structure_bounds = Region::new(chunk_bounds.min - V2::new(1, 1),
                                           chunk_bounds.max + V2::new(0, 1));
        self.renderer.update_structure_geometry(&self.data, &self.structures, structure_bounds);

        // Light from any adjacent chunk can extend into the current one.
        let light_bounds = Region::new(chunk_bounds.min - V2::new(1, 1),
                                       chunk_bounds.max + V2::new(1, 1));
        self.renderer.update_light_geometry(&self.data, &self.structures, light_bounds);

        // Entities can extend in any direction from their reference point.
        let entity_bounds = Region::new(chunk_bounds.min - V2::new(1, 1),
                                        chunk_bounds.max + V2::new(1, 1));
        self.renderer.update_entity_geometry(&self.data, &self.entities, entity_bounds, scene.now);

        // Also refresh the UI buffer.
        let (geom, cursor) = self.with_ui_dyn(|ui, dyn| {
            let geom = ui.generate_geom(dyn);
            let cursor = ui.get_cursor(dyn);
            (geom, cursor)
        });
        self.renderer.load_ui_geometry(&geom);
        if cursor != self.last_cursor {
            self.platform.set_cursor(cursor);
            self.last_cursor = cursor;
        }
    }

    pub fn render_frame(&mut self, now: Time) {
        let pos =
            if let Some(e) = self.pawn_id.and_then(|id| self.entities.get(id)) {
                e.pos(now)
            } else {
                V3::new(4096, 4096, 0)
            };
        let scene = Scene::new(now, self.window_size, self.view_size, pos);

        self.entities.apply_updates(scene.now);
        self.prepare(&scene);

        self.renderer.update_framebuffers(self.platform.gl(), &scene);
        self.renderer.render(&scene);
    }


    // Misc

    pub fn resize_window(&mut self, size: (u16, u16)) {
        let scale_config = self.platform.config().get_int(ConfigKey::ScaleWorld) as i16;
        let scale =
            if scale_config != 0 { scale_config }
            else { self.calc_scale(size) };
        println!("set scale to {} ({:?})", scale, size);

        let view_size =
            if scale > 0 {
                let shrink = scale as u16;
                ((size.0 + shrink - 1) / shrink,
                 (size.1 + shrink - 1) / shrink)
            } else {
                let grow = (-scale) as u16;
                (size.0 * grow,
                 size.1 * grow)
            };

        self.window_size = size;
        self.view_size = view_size;
    }

    fn calc_scale(&self, size: (u16, u16)) -> i16 {
        let (w, h) = size;
        let max_dim = cmp::max(w, h);
        const TARGET: u16 = 1024;
        if max_dim > TARGET {
            ((max_dim + TARGET / 2) / TARGET) as i16
        } else {
            -(((TARGET + max_dim / 2) / max_dim) as i16)
        }
    }

    pub fn bench(&mut self) {
        let mut counter = 0;
        for i in 0 .. 10000 {
            //let geom = self.ui.generate_geom(&self.inventories);
            //counter += geom.len();
            //self.renderer.load_ui_geometry(&geom);
        }
        println!("counter {}", counter);
    }
}


pub trait ClientObj {
    fn data(&self) -> &Data;

    fn inventories(&self) -> &Inventories;

    fn platform(&mut self) -> &mut PlatformObj;
    fn ui(&mut self) -> &mut UI;
}

impl<'d, P: Platform> ClientObj for Client<'d, P> {
    fn data(&self) -> &Data { &self.data }
    fn inventories(&self) -> &Inventories { &self.inventories }
    fn platform(&mut self) -> &mut PlatformObj { &mut self.platform }
    fn ui(&mut self) -> &mut UI { &mut self.ui }
}


fn rle16Decode(input: &[u16], output: &mut [u16]) {
    let mut i = 0;
    let mut j = 0;
    while i < input.len() {
        let x = input[i];
        i += 1;

        if x & 0xf000 == 0 {
            output[j] = x;
            j += 1;
        } else {
            let count = x & 0x0fff;
            let value = input[i];
            i += 1;

            for _ in 0..count {
                output[j] = value;
                j += 1;
            }
        }
    }
}
