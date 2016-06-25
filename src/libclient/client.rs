use std::prelude::v1::*;
use std::boxed::FnBox;
use std::cmp;

use platform::{Platform, PlatformObj};
use platform::{Config, ConfigKey};
use platform::Cursor;
use platform::gl::Context as GlContext;
use util;

use graphics::types::LocalChunks;
use graphics::types::HAS_LIGHT;
use graphics::renderer::Renderer;

use physics;
use physics::{CHUNK_SIZE, CHUNK_BITS, TILE_SIZE};
use physics::Shape;
use physics::v3::{V3, V2, Vn, scalar, Region};
use common_movement::InputBits;

use Time;
use data::Data;
use debug::Debug;
use entity::{self, Entities, EntityId};
use graphics::renderer::Scene;
use graphics::renderer::ONESHOT_MODULUS;
use graphics::types::StructureTemplate;
use inventory::{Inventories, Item, InventoryId};
use misc::Misc;
use predict::{Predictor, Activity};
use structures::Structures;
use terrain::TerrainShape;
use terrain::{LOCAL_SIZE, LOCAL_BITS};
use ui::{UI, Dyn};
use ui::input::{KeyAction, Modifiers, KeyEvent, EventStatus};


pub struct Client<'d, P: Platform> {
    data: &'d Data,
    platform: P,

    chunks: Box<LocalChunks>,
    terrain_shape: Box<TerrainShape>,
    structures: Structures,
    entities: Entities,
    inventories: Inventories,
    predictor: Predictor,
    misc: Misc,
    debug: Debug,

    ui: UI,

    renderer: Renderer<P::GL>,

    pawn_id: Option<EntityId>,
    default_camera_pos: V3,
    window_size: (u16, u16),
    view_size: (u16, u16),

    last_cursor: Cursor,
}

impl<'d, P: Platform> Client<'d, P> {
    pub fn new(data: &'d Data, platform: P) -> Client<'d, P> {
        let mut platform = platform;
        let mut renderer = Renderer::new(platform.gl());
        if platform.config().get_int(ConfigKey::RenderNames) == 0 {
            renderer.render_names = false;
        }

        let mut c = Client {
            data: data,
            platform: platform,

            chunks: box [[0; 1 << (3 * CHUNK_BITS)]; 1 << (2 * LOCAL_BITS)],
            terrain_shape: box TerrainShape::new(),
            structures: Structures::new(),
            entities: Entities::new(),
            inventories: Inventories::new(),
            predictor: Predictor::new(),
            misc: Misc::new(),
            debug: Debug::new(),

            ui: UI::new(),

            renderer: renderer,

            pawn_id: None,
            default_camera_pos: V3::new(4096, 4096, 0),
            window_size: (640, 480),
            view_size: (640, 480),

            last_cursor: Cursor::Normal,
        };

        c.misc.hotbar.init(c.platform.config(), &c.data);
        c.ui.root.init(c.platform.config());

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
        self.renderer.invalidate_structure_light_geometry();
    }

    pub fn reset_renderer(&mut self) {
        self.platform.gl().havoc();
        self.renderer = Renderer::new(self.platform.gl());
        self.renderer.invalidate_terrain_geometry();
        self.renderer.invalidate_structure_geometry();
        self.renderer.invalidate_structure_light_geometry();
    }

    // Terrain chunk tracking

    pub fn load_terrain_chunk(&mut self, cpos: V2, data: &[u16]) {
        // Update self.chunks
        let bounds = Region::new(scalar(0), scalar(LOCAL_SIZE));
        let blocks = &mut self.chunks[bounds.index(cpos)];
        rle16_decode(data, blocks);

        // Update self.terrain_shape
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE)) +
                           (cpos * scalar(CHUNK_SIZE)).extend(0);
        let block_data = self.data.blocks();
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
        let shape = &self.data.template_shapes()[base .. base + bounds.volume() as usize];
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
                            pixel_pos: V3,
                            template_id: u32,
                            oneshot_start: Time) {
        // Update self.structures
        const MASK: i32 = LOCAL_SIZE * CHUNK_SIZE - 1;
        let tile_pos = pixel_pos.div_floor(scalar(TILE_SIZE)) & scalar(MASK);
        let pos = (tile_pos.x as u8,
                   tile_pos.y as u8,
                   tile_pos.z as u8);
        let oneshot_start = (oneshot_start % ONESHOT_MODULUS) as u16;
        self.structures.insert(id, pos, template_id, oneshot_start);

        // Update self.terrain_cache
        let t = self.data.template(template_id);
        self.add_structure_shape(t, pos);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
        if t.flags.contains(HAS_LIGHT) {
            self.renderer.invalidate_structure_light_geometry();
        }
    }

    pub fn structure_gone(&mut self,
                          id: u32) {
        // Update self.structures
        let s = self.structures.remove(id);

        // Update self.terrain_cache
        let t = self.data.template(s.template_id);
        self.remove_structure_shape(t, s.pos);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
        if t.flags.contains(HAS_LIGHT) {
            self.renderer.invalidate_structure_light_geometry();
        }
    }

    pub fn structure_replace(&mut self,
                             id: u32,
                             template_id: u32,
                             oneshot_start: i32) {
        let (pos, old_t) = {
            let s = &self.structures[id];
            (s.pos,
             self.data.template(s.template_id))
        };
        let new_t = self.data.template(template_id);

        // Update self.structures
        let oneshot_start = (oneshot_start % ONESHOT_MODULUS) as u16;
        self.structures.replace(id, template_id, oneshot_start);

        // Update self.terrain_cache
        self.remove_structure_shape(old_t, pos);
        self.add_structure_shape(new_t, pos);

        // Invalidate cached geometry
        self.renderer.invalidate_structure_geometry();
        if old_t.flags.contains(HAS_LIGHT) || new_t.flags.contains(HAS_LIGHT) {
            self.renderer.invalidate_structure_light_geometry();
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

    pub fn entity_motion_start(&mut self,
                               id: EntityId,
                               start_time: Time,
                               start_pos: V3,
                               velocity: V3,
                               anim: u16) {
        self.entities.schedule_motion_start(id, start_time, start_pos, velocity, anim);

        if Some(id) == self.pawn_id {
            self.predictor.canonical_motion_update(entity::Update::MotionStart(
                    start_time, start_pos, velocity, anim));
        }
    }

    pub fn entity_motion_end(&mut self,
                             id: EntityId,
                             end_time: Time) {
        self.entities.schedule_motion_end(id, end_time);

        if Some(id) == self.pawn_id {
            self.predictor.canonical_motion_update(entity::Update::MotionEnd(end_time));
        }
    }

    pub fn entity_activity_icon(&mut self,
                                id: EntityId,
                                anim_id: u16) {
        if anim_id == self.data.activity_none_anim() {
            self.entities.set_activity_anim(id, None);
        } else {
            self.entities.set_activity_anim(id, Some(anim_id));
        }
    }

    pub fn set_pawn_id(&mut self,
                       id: EntityId) {
        self.pawn_id = Some(id);
    }

    pub fn clear_pawn_id(&mut self) {
        self.pawn_id = None;
    }

    pub fn set_default_camera_pos(&mut self, pos: V3) {
        self.default_camera_pos = pos;
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
                           &self.inventories,
                           &self.misc.hotbar,
                           &self.debug);
        f(&mut self.ui, dyn)
    }

    pub fn input_key(&mut self, code: u8, mods: u8) -> bool {
        let status =
            if let Some(key) = KeyAction::from_code(code) {
                let mods = Modifiers::from_bits_truncate(mods);
                let evt = KeyEvent::new(key, mods);
                self.with_ui_dyn(|ui, dyn| ui.handle_key(evt, dyn))
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

    pub fn open_ability_dialog(&mut self) {
        use ui::dialogs::AnyDialog;
        self.ui.root.dialog.inner = AnyDialog::ability();
    }

    pub fn open_container_dialog(&mut self, inv0: InventoryId, inv1: InventoryId) {
        use ui::dialogs::AnyDialog;
        self.ui.root.dialog.inner = AnyDialog::container(inv0, inv1);
    }

    pub fn get_active_item(&self) -> u16 {
        self.misc.hotbar.active_item().unwrap_or(0)
    }

    pub fn get_active_ability(&self) -> u16 {
        self.misc.hotbar.active_ability().unwrap_or(0)
    }


    // Physics

    pub fn feed_input(&mut self, time: Time, bits: u16) {
        self.predictor.input(time, InputBits::from_bits(bits).expect("invalid input bits"));
    }

    pub fn processed_inputs(&mut self, time: Time, count: u16) {
        self.predictor.processed_inputs(time, count as usize);
    }

    pub fn activity_change(&mut self, activity: u8) {
        let activity = match activity {
            0 => Activity::Walk,
            //1 => Activity::Fly,
            2 => Activity::Emote,
            3 => Activity::Work,
            _ => panic!("invalid activity value: {}", activity),
        };
        self.predictor.activity_update(activity);
    }


    // Graphics

    fn prepare(&mut self, scene: &Scene, future: Time) {
        self.renderer.update_framebuffers(self.platform.gl(), &scene);

        let bounds = Region::sized(scene.camera_size) + scene.camera_pos;
        let tile_bounds = bounds.div_round_signed(TILE_SIZE);
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
        self.renderer.update_structure_light_geometry(&self.data, &self.structures, light_bounds);

        // Entities can extend in any direction from their reference point.
        {
            let pawn_id = self.pawn_id;
            let predictor = &self.predictor;
            self.entities.update_z_order(|e| {
                let mut pos =
                    if Some(e.id) == pawn_id {
                        predictor.motion().pos(future)
                    } else {
                        e.pos(scene.now)
                    };
                if pos.y < scene.camera_pos.y - CHUNK_SIZE * TILE_SIZE {
                    pos.y += CHUNK_SIZE * TILE_SIZE * LOCAL_SIZE;
                }
                pos.y - pos.z
            });
        }
        let entity_bounds = Region::new(chunk_bounds.min - V2::new(1, 1),
                                        chunk_bounds.max + V2::new(1, 1));
        self.renderer.update_entity_geometry(&self.data,
                                             &self.entities,
                                             &self.predictor,
                                             entity_bounds,
                                             scene.now,
                                             future,
                                             self.pawn_id);

        // Also refresh the UI buffer.
        let (geom, special, cursor) = self.with_ui_dyn(|ui, dyn| {
            let (geom, special) = ui.generate_geom(dyn);
            let cursor = ui.get_cursor(dyn);
            (geom, special, cursor)
        });
        self.renderer.load_ui_geometry(&geom);
        self.renderer.set_ui_special(special);
        if cursor != self.last_cursor {
            self.platform.set_cursor(cursor);
            self.last_cursor = cursor;
        }

        // Update the cavern map
        // TODO: cache this
        let mut grid = box [physics::floodfill::flags::Flags::empty(); 96 * 96];
        let slice_offset = scene.slice_center.reduce() - scalar::<V2>(48);
        // `grid_bounds` is a region the size of the grid, centered at slice_center.
        let grid_bounds = Region::sized(scalar(96)) + slice_offset;
        let slice_bounds = Region::new(tile_bounds.min,
                                       tile_bounds.max + V2::new(0, 16));
        physics::floodfill::floodfill(scene.slice_center,
                                      slice_bounds.intersect(grid_bounds),
                                      &*self.terrain_shape,
                                      &mut *grid,
                                      grid_bounds);
        self.renderer.load_cavern_map(&*grid);
    }

    pub fn render_frame(&mut self, now: Time, future: Time) {
        self.debug.record_interval(now);
        let day_time = self.misc.day_night.time_of_day(now);
        self.debug.day_time = day_time;
        self.debug.day_phase = self.misc.day_night.phase_delta(&self.data, day_time).0;

        self.predictor.update(future, &*self.terrain_shape, &self.data);

        let pos =
            if self.pawn_id.is_some() {
                // TODO: hardcoded constant based on entity size
                self.predictor.motion().pos(future) + V3::new(16, 16, 0)
            } else {
                self.default_camera_pos
            };
        // Wrap `pos` to 2k .. 6k region
        let pos = util::wrap_base(pos, V3::new(2048, 2048, 0));
        self.debug.pos = pos;

        let ambient_light =
            if self.misc.plane_is_dark { (0, 0, 0, 0) }
            else { self.misc.day_night.ambient_light(&self.data, now) };
        let cursor_pos =
            if self.misc.show_cursor && self.pawn_id.is_some() {
                calc_cursor_pos(&self.data, pos, self.predictor.motion().anim_id)
            } else {
                None
            };
        let scene = Scene::new(now,
                               self.window_size,
                               self.view_size,
                               pos,
                               ambient_light,
                               cursor_pos);

        self.entities.apply_updates(scene.now);
        self.prepare(&scene, future);

        self.renderer.render(&scene);
    }


    // Misc

    pub fn debug_record(&mut self, frame_time: Time, ping: u32) {
        self.debug.record_frame_time(frame_time);
        self.debug.ping = ping;
    }

    pub fn init_day_night(&mut self, base_time: Time, cycle_ms: Time) {
        self.misc.day_night.init(base_time, cycle_ms);
    }

    pub fn set_plane_flags(&mut self, flags: u32) {
        self.misc.plane_is_dark = flags != 0;
    }

    pub fn toggle_cursor(&mut self) {
        self.misc.show_cursor = !self.misc.show_cursor;
    }

    pub fn calc_scale(&self, size: (u16, u16)) -> i16 {
        let scale_config = self.platform.config().get_int(ConfigKey::ScaleWorld) as i16;
        if scale_config != 0 {
            return scale_config;
        }

        let (w, h) = size;
        let max_dim = cmp::max(w, h);
        const TARGET: u16 = 1024;
        if max_dim > TARGET {
            ((max_dim + TARGET / 2) / TARGET) as i16
        } else {
            -(((TARGET + max_dim / 2) / max_dim) as i16)
        }
    }

    pub fn resize_window(&mut self, size: (u16, u16)) {
        let scale = self.calc_scale(size);
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

    pub fn ponyedit_render(&mut self, appearance: u32) {
        let mut entities = Entities::new();
        entities.insert(0, appearance, None);
        let anim = self.data().editor_anim();
        entities.ponyedit_hack(0, anim, self.default_camera_pos);
        println!("created entity hack with app {:x}", appearance);

        let scene = Scene::new(0,
                               self.window_size,
                               self.view_size,
                               self.default_camera_pos + V3::new(16, 16, 0),
                               (255, 255, 255, 255),
                               None);
        let cpos = self.default_camera_pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

        self.renderer.update_framebuffers(self.platform.gl(), &scene);
        entities.update_z_order(|_| 0);
        self.renderer.update_entity_geometry(&self.data,
                                             &entities,
                                             &self.predictor,
                                             Region::new(scalar(-1), scalar(1)) + cpos,
                                             0,
                                             0,
                                             None);
        self.renderer.render_ponyedit_hack(&scene);
    }

    pub fn bench(&mut self) {
        #![allow(warnings)]
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

    fn handle_hotbar_assign(&mut self, idx: u8, item_id: u16, is_ability: bool);
    fn handle_hotbar_drop(&mut self,
                          src_inv: InventoryId,
                          src_slot: usize,
                          dest_slot: u8);
    fn handle_hotbar_select(&mut self, idx: u8);
}

impl<'d, P: Platform> ClientObj for Client<'d, P> {
    fn data(&self) -> &Data { &self.data }
    fn inventories(&self) -> &Inventories { &self.inventories }
    fn platform(&mut self) -> &mut PlatformObj { &mut self.platform }
    fn ui(&mut self) -> &mut UI { &mut self.ui }

    fn handle_hotbar_assign(&mut self, idx: u8, item_id: u16, is_ability: bool) {
        self.misc.hotbar.set_slot(&self.data,
                                  self.platform.config_mut(),
                                  idx,
                                  item_id,
                                  is_ability);
    }

    fn handle_hotbar_drop(&mut self,
                          src_inv: InventoryId,
                          src_slot: usize,
                          dest_slot: u8) {
        let item_id = match self.inventories.get(src_inv) {
            Some(x) => x.items[src_slot].id,
            None => return,
        };
        // TODO: hack
        let is_ability = Some(src_inv) == self.inventories.ability_id();
        self.handle_hotbar_assign(dest_slot, item_id, is_ability);
    }

    fn handle_hotbar_select(&mut self, idx: u8) {
        self.misc.hotbar.select(self.platform.config_mut(), idx);
    }
}


fn rle16_decode(input: &[u16], output: &mut [u16]) {
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

static DIRS: [V2; 8] = [
    V2 { x:  1, y:  0 },
    V2 { x:  1, y:  1 },
    V2 { x:  0, y:  1 },
    V2 { x: -1, y:  1 },
    V2 { x: -1, y:  0 },
    V2 { x: -1, y: -1 },
    V2 { x:  0, y: -1 },
    V2 { x:  1, y: -1 },
];

fn calc_cursor_pos(data: &Data, pos: V3, anim: u16) -> Option<V2> {
    let dir = match data.anim_dir(anim) {
        Some(x) => x,
        None => return None,
    };

    // TODO: need z + 16 adjustment to work right on stairs
    let tile = pos.div_floor(scalar(TILE_SIZE));
    let pos = tile + DIRS[dir as usize].extend(0);
    Some(V2::new(pos.x, pos.y - pos.z) * scalar(TILE_SIZE) + scalar(TILE_SIZE / 2))
}
