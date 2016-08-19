use std::prelude::v1::*;
use types::*;
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
use common::Gauge;
use common_movement::InputBits;
use common_proto::game::{Request, Response};
use common_proto::types::LocalTime;

use data::Data;
use debug::Debug;
use entity::{self, Entity, Entities};
use graphics::renderer::Scene;
use graphics::renderer::ONESHOT_MODULUS;
use graphics::types::StructureTemplate;
use input::{Key, Modifiers, KeyEvent, Button, ButtonEvent, EventStatus};
use inventory::{Inventories, Item};
use misc::Misc;
use pawn::{PawnInfo, Activity};
use structures::Structures;
use terrain::TerrainShape;
use terrain::{LOCAL_SIZE, LOCAL_BITS};
use timing::{Timing, TICK_MS};
use ui::{UI, Dyn};
use ui::dialogs::AnyDialog;


pub struct Client<'d, P: Platform> {
    data: &'d Data,
    platform: P,

    chunks: Box<LocalChunks>,
    terrain_shape: Box<TerrainShape>,
    structures: Structures,
    entities: Entities,
    inventories: Inventories,
    misc: Misc,
    debug: Debug,
    timing: Timing,
    pawn: PawnInfo,

    ui: UI,

    renderer: Renderer<P::GL>,

    default_camera_pos: V3,
    window_size: (u16, u16),
    view_size: (u16, u16),
    ui_scale: u16,

    last_cursor: Cursor,
    cur_input: InputBits,
}

// Helper macro for chaining event handlers.
macro_rules! try_handle {
    ($slf:ident, $e:expr) => {
        let status = $e;
        if let EventStatus::Unhandled = status {
            // Do nothing
        } else {
            return $slf.process_event_status(status);
        }
    };
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
            misc: Misc::new(),
            debug: Debug::new(),
            timing: Timing::new(),
            pawn: PawnInfo::new(),

            ui: UI::new(),

            renderer: renderer,

            default_camera_pos: V3::new(4096, 4096, 0),
            window_size: (640, 480),
            view_size: (640, 480),
            ui_scale: 1,

            last_cursor: Cursor::Normal,
            cur_input: InputBits::empty(),
        };

        c.misc.hotbar.init(c.platform.config(), &c.data);
        c.ui.root.init(c.platform.config());

        c
    }


    // Main entry point

    pub fn handle_message(&mut self, resp: Response) {
        match resp {
            Response::TerrainChunk(idx, data) => {
                let cpos = V2::new(idx as i32 % LOCAL_SIZE,
                                   idx as i32 / LOCAL_SIZE);
                self.load_terrain_chunk(cpos, &data);
            },

            Response::Pong(_cookie, _now) => error!("NYI: libclient Pong"),

            Response::Init(pawn_id, now, day_night_base, day_night_ms) => {
                self.set_pawn_id(pawn_id);
                self.init_timing(now.unwrap());
                self.init_day_night(now.unwrap(), day_night_base as Time, day_night_ms as Time);
            },

            Response::KickReason(_msg) => error!("NYI: libclient KickReason"),

            Response::UnloadChunk(_idx) => {},   // TODO (currently no-op)

            Response::OpenCrafting(_kind, station, inventory) =>
                self.open_crafting_dialog(inventory, station),

            Response::OpenDialog(which, args) => match which {
                1 => {
                    assert!(args.len() == 2);
                    let iid1 = InventoryId(args[0]);
                    let iid2 = InventoryId(args[1]);
                    self.open_container_dialog(iid1, iid2);
                },
                _ => error!("NYI: libclient OpenDialog({})", which),
            },

            Response::ChatUpdate(_msg) => error!("NYI: libclient ChatUpdate"),

            Response::EntityAppear(id, appearance, name) => {
                let opt_name = if name != "" { Some(name) } else { None };
                self.entity_appear(id, appearance, opt_name);
            },

            Response::EntityGone(id, time) =>
                self.entity_gone(id, time.unwrap()),

            Response::StructureAppear(id, template, pos) =>
                self.structure_appear(id, pos.unwrap(), template),

            Response::StructureGone(id) =>
                self.structure_gone(id),

            Response::MainInventory(id) =>
                self.inventories.set_main_id(id),

            Response::AbilityInventory(id) =>
                self.inventories.set_ability_id(id),

            Response::PlaneFlags(flags) =>
                self.set_plane_flags(flags),

            Response::GetInteractArgs(_dialog, _arg) =>
                error!("NYI: libclient GetInteractArgs"),

            Response::GetUseItemArgs(_item, _dialog, _arg) =>
                error!("NYI: libclient GetUseItemArgs"),

            Response::GetUseAbilityArgs(_ability, _dialog, _arg) =>
                error!("NYI: libclient GetUseAbilityArgs"),

            Response::SyncStatus(_status) => error!("NYI: libclient SyncStatus"),

            Response::StructureReplace(id, template) =>
                self.structure_replace(id, template),

            Response::InventoryUpdate(id, slot, raw_item) => {
                let (_, qty, item_id) = raw_item;
                let item = Item::new(item_id, qty);
                self.inventory_update(id, slot as usize, item);
            },

            Response::InventoryAppear(id, raw_items) => {
                let items = raw_items.into_iter()
                    .map(|(_, qty, item_id)| Item::new(item_id, qty))
                    .collect::<Vec<_>>().into_boxed_slice();
                self.inventory_appear(id, items);
            },

            Response::InventoryGone(id) =>
                self.inventory_gone(id),

            Response::EntityMotionStart(id, start_pos, start_time, velocity, anim) =>
                self.entity_motion_start(id,
                                         start_time.unwrap(),
                                         start_pos.unwrap(),
                                         velocity.unwrap(),
                                         anim),

            Response::EntityMotionEnd(id, end_time) =>
                self.entity_motion_end(id,
                                       end_time.unwrap()),

            Response::EntityMotionStartEnd(
                    id, start_pos, start_time, velocity, anim, end_time) => {
                self.entity_motion_start(id,
                                         start_time.unwrap(),
                                         start_pos.unwrap(),
                                         velocity.unwrap(),
                                         anim);
                self.entity_motion_end(id,
                                       end_time.unwrap());
            },

            Response::ActivityChange(activity) =>
                self.activity_change(activity),

            Response::InitNoPawn(camera_pos, now, day_night_base, day_night_ms) => {
                self.set_default_camera_pos(camera_pos.unwrap());
                self.init_timing(now.unwrap());
                self.init_day_night(now.unwrap(), day_night_base as Time, day_night_ms as Time);
            },

            Response::OpenPonyEdit(_name) => error!("NYI: libclient OpenPonyEdit"),

            Response::EntityActivityIcon(id, icon) =>
                self.entity_activity_icon(id, icon),

            Response::CancelDialog(()) =>
                self.close_dialog(),

            Response::EnergyUpdate(cur, max, rate, time) =>
                self.energy_update(cur as i32, max as i32, rate, time.unwrap()),

            Response::ResetMotion(()) =>
                self.pawn.reset_motion(),
        }
    }

    // Resetting client state

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

        self.pawn.clear_id(&mut self.entities);
        self.pawn = PawnInfo::new();

        self.misc.reset();

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
                            id: StructureId,
                            pixel_pos: V3,
                            template_id: TemplateId) {
        // Update self.structures
        const MASK: i32 = LOCAL_SIZE * CHUNK_SIZE - 1;
        let tile_pos = pixel_pos.div_floor(scalar(TILE_SIZE)) & scalar(MASK);
        let pos = (tile_pos.x as u8,
                   tile_pos.y as u8,
                   tile_pos.z as u8);
        // TODO: get a timestamp from the server, instead of guessing now()
        let oneshot_start = (self.now() % ONESHOT_MODULUS) as u16;
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
                          id: StructureId) {
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
                             id: StructureId,
                             template_id: TemplateId) {
        let (pos, old_t) = {
            let s = &self.structures[id];
            (s.pos,
             self.data.template(s.template_id))
        };
        let new_t = self.data.template(template_id);

        // Update self.structures
        // TODO: get a timestamp from the server, instead of guessing now()
        let oneshot_start = (self.now() % ONESHOT_MODULUS) as u16;
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
        if self.pawn.is(id) {
            self.pawn.on_create(&mut self.entities);
        }
    }

    pub fn entity_gone(&mut self,
                       id: EntityId,
                       _time: u16) {
        // TODO: use the time
        self.entities.remove(id);
    }

    pub fn entity_motion_start(&mut self,
                               id: EntityId,
                               start_time: u16,
                               start_pos: V3,
                               velocity: V3,
                               anim: u16) {
        let start_time = self.decode_time(start_time);
        if self.pawn.is(id) {
            self.pawn.server_update(entity::Update::MotionStart(
                    start_time, start_pos, velocity, anim));
        } else {
            self.entities.schedule_motion_start(id, start_time, start_pos, velocity, anim);
        }
    }

    pub fn entity_motion_end(&mut self,
                             id: EntityId,
                             end_time: u16) {
        let end_time = self.decode_time(end_time);
        if self.pawn.is(id) {
            self.pawn.server_update(entity::Update::MotionEnd(end_time));
        } else {
            self.entities.schedule_motion_end(id, end_time);
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
        self.pawn.set_id(id, &mut self.entities);
    }

    pub fn clear_pawn_id(&mut self) {
        self.pawn.clear_id(&mut self.entities);
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
        if self.inventories.main_id() == Some(id) {
            if let Some(inv) = self.inventories.main_inventory() {
                if slot < inv.len() {
                    let old = inv.items[slot];
                    let now = self.now();
                    if old.id == item.id {
                        let delta = item.quantity as i16 - old.quantity as i16;
                        self.misc.inv_changes.add(now, item.id, delta);
                    } else {
                        self.misc.inv_changes.add(now, old.id, -(old.quantity as i16));
                        self.misc.inv_changes.add(now, item.id, item.quantity as i16);
                    }
                }
            }
        }

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

    fn with_ui_dyn<F: FnOnce(&mut UI, &Dyn) -> R, R>(&mut self, f: F) -> R {
        let c = self.ui_scale;
        let sx = (self.view_size.0 + c - 1) / c;
        let sy = (self.view_size.1 + c - 1) / c;

        // TODO: should take the time as an argument instead of calling now()
        let dyn = Dyn::new((sx, sy),
                           self.now(),
                           &self.data,
                           &self.inventories,
                           &self.misc,
                           &self.debug);
        f(&mut self.ui, &dyn)
    }

    pub fn input_key_down(&mut self, code: u8, mods: u8) -> bool {
        if let Some(key) = Key::from_code(code) {
            let mods = Modifiers::from_bits_truncate(mods);
            let evt = KeyEvent::new(key, mods);
            try_handle!(self, self.with_ui_dyn(|ui, dyn| ui.handle_key(evt, dyn)));
            if self.ui.root.dialog.inner.is_none() {
                try_handle!(self, self.handle_ui_key(evt));
                try_handle!(self, self.handle_key_down(evt));
            }
        }

        self.process_event_status(EventStatus::Unhandled)
    }

    pub fn input_key_up(&mut self, code: u8, mods: u8) -> bool {
        if let Some(key) = Key::from_code(code) {
            let mods = Modifiers::from_bits_truncate(mods);
            let evt = KeyEvent::new(key, mods);
            if self.ui.root.dialog.inner.is_none() {
                try_handle!(self, self.handle_key_up(evt));
            }
        }

        self.process_event_status(EventStatus::Unhandled)
    }

    fn handle_ui_key(&mut self, evt: KeyEvent) -> EventStatus {
        use input::Key::*;

        match evt.code {
            ToggleCursor => {
                self.misc.show_cursor = !self.misc.show_cursor;
            },
            OpenInventory => {
                self.ui.root.dialog.inner = AnyDialog::inventory();
            },
            OpenAbilities => {
                self.ui.root.dialog.inner = AnyDialog::ability();
            },
            _ => return EventStatus::Unhandled,
        }

        EventStatus::Handled
    }

    fn handle_key_down(&mut self, evt: KeyEvent) -> EventStatus {
        use common_movement::*;
        use input::Key::*;

        // Update input bits
        let old_input = self.cur_input;
        let bits =
            match evt.code {
                MoveLeft =>     INPUT_LEFT,
                MoveRight =>    INPUT_RIGHT,
                MoveUp =>       INPUT_UP,
                MoveDown =>     INPUT_DOWN,
                Run =>          INPUT_RUN,
                Interact |
                UseItem |
                UseAbility |
                Cancel =>
                    // Don't hold if the character is already moving
                    if (old_input & INPUT_DIR_MASK).is_empty() { INPUT_HOLD }
                    else { InputBits::empty() },
                DebugLogSwitch => {
                    ::std::log_level(5);
                    trace!("\n\n\n === TRACING ENABLED ===");
                    return EventStatus::Unhandled;
                },
                _ => return EventStatus::Unhandled,
            };
        self.cur_input.insert(bits);
        self.send_input(old_input);

        EventStatus::Handled
    }

    fn handle_key_up(&mut self, evt: KeyEvent) -> EventStatus {
        use common_movement::*;
        use input::Key::*;

        // Update input bits
        let old_input = self.cur_input;
        let bits =
            match evt.code {
                MoveLeft =>     INPUT_LEFT,
                MoveRight =>    INPUT_RIGHT,
                MoveUp =>       INPUT_UP,
                MoveDown =>     INPUT_DOWN,
                Run =>          INPUT_RUN,
                Interact |
                UseItem |
                UseAbility |
                Cancel =>       INPUT_HOLD,
                DebugLogSwitch => {
                    trace!(" === TRACING DISABLED ===\n\n\n");
                    ::std::log_level(2);
                    return EventStatus::Unhandled;
                },
                _ => return EventStatus::Unhandled,
            };
        self.cur_input.remove(bits);
        self.send_input(old_input);

        // Also send action command, if needed
        let time = self.predict_arrival(0);
        match evt.code {
            Interact =>
                self.platform.send_message(
                    Request::Interact(LocalTime::from_global_32(time))),
            UseItem =>
                self.platform.send_message(
                    Request::UseItem(LocalTime::from_global_32(time),
                                     self.misc.hotbar.active_item().unwrap_or(0))),
            UseAbility =>
                self.platform.send_message(
                    Request::UseAbility(LocalTime::from_global_32(time),
                                        self.misc.hotbar.active_ability().unwrap_or(0))),
            _ => {},
        }

        EventStatus::Handled
    }

    fn send_input(&mut self, old: InputBits) {
        let new = self.cur_input;
        use common_movement::INPUT_DIR_MASK;
        if new != old && !((new | old) & INPUT_DIR_MASK).is_empty() {
            self.pawn.set_input(new);
        }
    }

    fn convert_mouse_pos(&self, pos: V2) -> V2 {
        let x = pos.x * self.view_size.0 as i32 / self.window_size.0 as i32;
        let y = pos.y * self.view_size.1 as i32 / self.window_size.1 as i32;
        V2::new(x, y) / scalar(self.ui_scale as i32)
    }

    pub fn input_mouse_move(&mut self, pos: V2) -> bool {
        let pos = self.convert_mouse_pos(pos);
        let status = self.with_ui_dyn(|ui, dyn| ui.handle_mouse_move(pos, dyn));
        self.process_event_status(status)
    }

    pub fn input_mouse_down(&mut self, pos: V2, button: u8, mods: u8) -> bool {
        let status =
            if let Some(button) = Button::from_code(button) {
                let pos = self.convert_mouse_pos(pos);
                let mods = Modifiers::from_bits_truncate(mods);
                let evt = ButtonEvent::new(button, mods);
                self.with_ui_dyn(|ui, dyn| ui.handle_mouse_down(pos, evt, dyn))
            } else {
                EventStatus::Unhandled
            };
        self.process_event_status(status)
    }

    pub fn input_mouse_up(&mut self, pos: V2, button: u8, mods: u8) -> bool {
        let status =
            if let Some(button) = Button::from_code(button) {
                let pos = self.convert_mouse_pos(pos);
                let mods = Modifiers::from_bits_truncate(mods);
                let evt = ButtonEvent::new(button, mods);
                self.with_ui_dyn(|ui, dyn| ui.handle_mouse_up(pos, evt, dyn))
            } else {
                EventStatus::Unhandled
            };
        self.process_event_status(status)
    }

    pub fn open_container_dialog(&mut self, inv0: InventoryId, inv1: InventoryId) {
        use ui::dialogs::AnyDialog;
        self.ui.root.dialog.inner = AnyDialog::container(inv0, inv1);
    }

    pub fn open_crafting_dialog(&mut self, inv: InventoryId, station: StructureId) {
        use ui::dialogs::AnyDialog;
        let template = self.structures[station].template_id;
        self.ui.root.dialog.inner = AnyDialog::crafting(inv, station, template);
    }

    pub fn close_dialog(&mut self) {
        use ui::dialogs::AnyDialog;
        self.ui.root.dialog.inner = AnyDialog::none();
    }


    // Physics

    pub fn activity_change(&mut self, activity: u8) {
        let activity = match activity {
            0 => Activity::Walk,
            //1 => Activity::Fly,
            2 => Activity::Emote,
            3 => Activity::Work,
            _ => panic!("invalid activity value: {}", activity),
        };
        self.pawn.set_activity(activity);
    }


    // Graphics

    fn prepare(&mut self, scene: &Scene) {
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
            self.entities.update_z_order(|e| {
                let mut pos = e.pos(scene.now);
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
                                             entity_bounds,
                                             scene.now);

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

    pub fn render_frame(&mut self) {
        let client_now = self.platform.get_time();
        // Only render ticks that are very likely to lie fully in the past.
        let now = self.timing.convert_confidence(client_now, -200);

        trace!(" --- begin frame @ {} ---", now);

        self.debug.record_interval(now);
        self.debug.ping = self.timing.get_ping() as u32;
        self.debug.ping_dev = self.timing.get_ping_dev() as u32;
        self.debug.delta_dev = self.timing.get_delta_dev() as u32;
        let day_time = self.misc.day_night.time_of_day(now);
        self.debug.day_time = day_time;
        self.debug.day_phase = self.misc.day_night.phase_delta(&self.data, day_time).0;

        // Update player position
        // This needs to happen before the camera position is set, in case the motion changed
        // between the previous frame and now.
        self.pawn.update_movement(now,
                                  self.data,
                                  &*self.terrain_shape,
                                  &mut self.platform,
                                  &mut self.entities);

        let pos =
            if let Some(pawn) = self.pawn() { pawn.pos(now) }
            else { self.default_camera_pos };
        // Wrap `pos` to 2k .. 6k region
        let pos = util::wrap_base(pos, V3::new(2048, 2048, 0));
        self.debug.pos = pos;
        trace!("camera pos: {:?}", pos);

        let ambient_light =
            if self.misc.plane_is_dark { (0, 0, 0, 0) }
            else { self.misc.day_night.ambient_light(&self.data, now) };
        let cursor_pos =
            if self.misc.show_cursor {
                self.pawn().and_then(|pawn| calc_cursor_pos(&self.data, pos, pawn.motion.anim_id))
            } else {
                None
            };
        let scene = Scene::new(now,
                               self.window_size,
                               self.view_size,
                               self.ui_scale,
                               pos,
                               ambient_light,
                               cursor_pos);

        self.entities.apply_updates(scene.now);
        self.prepare(&scene);

        self.renderer.render(&scene);
    }


    // Misc

    // TODO: use LocalTime
    fn decode_time(&self, server: u16) -> Time {
        let client = self.platform.get_time();
        self.timing.decode(client, server)
    }

    fn now(&self) -> Time {
        let client = self.platform.get_time();
        self.timing.convert(client)
    }

    fn pawn(&self) -> Option<&Entity> {
        self.pawn.get(&self.entities)
    }

    pub fn debug_record(&mut self, frame_time: Time) {
        self.debug.record_frame_time(frame_time);
    }

    pub fn init_day_night(&mut self, now: u16, base_offset: Time, cycle_ms: Time) {
        // Compute the server time when the day/night cycle began
        let base_time = self.decode_time(now) - base_offset;
        self.misc.day_night.init(base_time, cycle_ms);
    }

    pub fn set_plane_flags(&mut self, flags: u32) {
        self.misc.plane_is_dark = flags != 0;
    }

    pub fn init_timing(&mut self, server: u16) {
        let client = self.platform.get_time();
        self.timing.init(client, server);

        self.misc.energy = Gauge::new(0, (1, 6), server as Time, 0, 240);
        self.pawn.init_time(server as Time);
    }

    pub fn handle_pong(&mut self, client_send: Time, client_recv: Time, server: u16) {
        self.timing.record_ping(client_send, client_recv, server);
    }

    pub fn predict_arrival(&mut self, extra_delay: Time) -> Time {
        let client = self.platform.get_time();
        let server = self.timing.predict_confidence(client, 200);
        (server + extra_delay + TICK_MS - 1) & !(TICK_MS - 1)
    }

    pub fn energy_update(&mut self, cur: i32, max: i32, rate: (i16, u16), time: u16) {
        let time = self.decode_time(time);
        self.misc.energy = Gauge::new(cur, rate, time, 0, max);
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

        self.ui_scale = match self.platform.config().get_int(ConfigKey::ScaleUI) {
            0 => 1,
            x => x as u16,
        };
    }

    pub fn ponyedit_render(&mut self, appearance: u32) {
        let mut entities = Entities::new();
        let zero_id = EntityId(0);
        entities.insert(zero_id, appearance, None);
        let anim = self.data().editor_anim();
        entities.ponyedit_hack(zero_id, anim, self.default_camera_pos);
        println!("created entity hack with app {:x}", appearance);

        let scene = Scene::new(0,
                               self.window_size,
                               self.view_size,
                               1,   // UI scale
                               self.default_camera_pos + V3::new(16, 16, 0),
                               (255, 255, 255, 255),
                               None);
        let cpos = self.default_camera_pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

        self.renderer.update_framebuffers(self.platform.gl(), &scene);
        entities.update_z_order(|_| 0);
        self.renderer.update_entity_geometry(&self.data,
                                             &entities,
                                             Region::new(scalar(-1), scalar(1)) + cpos,
                                             0);
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

    fn handle_hotbar_assign(&mut self, idx: u8, item_id: ItemId, is_ability: bool);
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
