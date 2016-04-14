use std::prelude::v1::*;
use std::boxed::FnBox;
use std::mem;

use platform::{Platform, PlatformObj};
use platform::gl::Context as GlContext;
use util;

use graphics::types::{BlockChunk, LocalChunks};
use graphics::types::HAS_LIGHT;
use graphics::renderer::Renderer;

use physics;
use physics::{CHUNK_SIZE, CHUNK_BITS};
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
use ui::UI;
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
        self.inventories.clear();

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

    pub fn input_key(&mut self, code: u8) -> bool {
        let status =
            if let Some(key) = KeyAction::from_code(code) {
                self.ui.handle_key(key, &self.inventories)
            } else {
                EventStatus::Unhandled
            };
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

    pub fn input_mouse_move(&mut self, pos: V2) -> bool {
        let status = self.ui.handle_mouse_move(pos, &self.inventories);
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

    pub fn prepare_geometry(&mut self, bounds: Region<V2>, now: Time) {
        self.platform.gl().havoc();

        // Terrain from the chunk below can cover the current one.
        let terrain_bounds = Region::new(bounds.min - V2::new(0, 0),
                                         bounds.max + V2::new(0, 1));
        self.renderer.update_terrain_geometry(&self.data, &self.chunks, terrain_bounds);

        // Structures from the chunk below can cover the current one, and also
        // structures from chunks above and to the left can extend into it.
        let structure_bounds = Region::new(bounds.min - V2::new(1, 1),
                                           bounds.max + V2::new(0, 1));
        self.renderer.update_structure_geometry(&self.data, &self.structures, structure_bounds);

        // Light from any adjacent chunk can extend into the current one.
        let light_bounds = Region::new(bounds.min - V2::new(1, 1),
                                       bounds.max + V2::new(1, 1));
        self.renderer.update_light_geometry(&self.data, &self.structures, light_bounds);

        // Also refresh the UI buffer.
        let geom = self.ui.generate_geom(&self.inventories);
        self.renderer.load_ui_geometry(&geom);


        self.entities.apply_updates(now);
        /*
        for (id, e) in self.entities.iter() {
            println!("entity {} at {:?}", id, e.pos(now));
        }
        */
    }

    pub fn get_terrain_geometry_buffer(&self) -> &<P::GL as GlContext>::Buffer {
        self.renderer.get_terrain_buffer()
    }

    pub fn get_structure_geometry_buffer(&self) -> &<P::GL as GlContext>::Buffer {
        self.renderer.get_structure_buffer()
    }

    pub fn get_light_geometry_buffer(&self) -> &<P::GL as GlContext>::Buffer {
        self.renderer.get_light_buffer()
    }

    pub fn get_ui_geometry_buffer(&self) -> &<P::GL as GlContext>::Buffer {
        self.renderer.get_ui_buffer()
    }

    pub fn test_render(&mut self, opcode: u32, scene: &Scene, tex_name: u32) {
        self.platform.gl().havoc();
        let tex = self.platform.gl().texture_import_HACK(tex_name, (1024, 1024));
        match opcode {
            0 => self.renderer.render_output(&tex),
            1 => self.renderer.render_terrain(scene, &tex),
            2 => self.renderer.render_structures(scene, &tex, false),
            3 => self.renderer.render_static_lights(scene, &tex),
            4 => self.renderer.render_structures(scene, &tex, true),
            _ => println!("bad opcode: {}", opcode),
        }
        mem::forget(tex);
    }


    // Misc

    pub fn bench(&mut self) {
        let mut counter = 0;
        for i in 0 .. 10000 {
            let geom = self.ui.generate_geom(&self.inventories);
            counter += geom.len();
            //self.renderer.load_ui_geometry(&geom);
        }
        println!("counter {}", counter);
    }
}


pub trait ClientObj {
    fn data(&self) -> &Data;
    fn platform(&mut self) -> &mut PlatformObj;
    fn ui(&mut self) -> &mut UI;
}

impl<'d, P: Platform> ClientObj for Client<'d, P> {
    fn data(&self) -> &Data { &self.data }
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
