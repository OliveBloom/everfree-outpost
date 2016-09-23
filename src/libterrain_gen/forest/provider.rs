use rand::{Rng, XorShiftRng};

use libphysics::CHUNK_SIZE;
use libserver_config::{Data, Storage, LootTables};
use libserver_types::*;

use {GenChunk, GenStructure};
use forest::context::{Context, TerrainGridPass, CaveRampsPass, CaveJunkPass, TreesPass};
use forest::terrain_grid::{self, Cell, FloorType};

pub struct Provider<'d> {
    data: &'d Data,
    loot_tables: &'d LootTables,
    ctx: Context<'d>,
}

impl<'d> Provider<'d> {
    pub fn new(data: &'d Data,
               loot_tables: &'d LootTables,
               storage: &'d Storage,
               rng: XorShiftRng) -> Provider<'d> {
        Provider {
            data: data,
            loot_tables: loot_tables,
            ctx: Context::new(storage, rng),
        }
    }

    pub fn context_mut(&mut self) -> &mut Context<'d> {
        &mut self.ctx
    }

    pub fn generate(&mut self, pid: Stable<PlaneId>, cpos: V2) -> GenChunk {
        let mut gc = GenChunk::new();
        let mut rng = self.ctx.get_rng(pid);

        let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE));
        let base = cpos * scalar(CHUNK_SIZE);
        let grid_bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));

        macro_rules! get_id {
            ($name:expr) => {
                match self.data.block_data.find_id($name) {
                    Some(x) => x,
                    None => { /*warn!("no such block: {}", $name);*/ 0 }
                }
            };
        }

        // TODO: move specs to its own TileSpecsPass
        let mut specs = Box::new(
            [[TileSpec::empty(); CHUNK_SIZE as usize * CHUNK_SIZE as usize]; 8]);

        {
            let t = self.ctx.result::<TerrainGridPass>((pid, cpos));

            // Apply terrain grid
            for layer in 0 .. CHUNK_SIZE / 2 {
                let z = layer * 2;
                let tl = &t.data[layer as usize];
                for pos in bounds.points() {
                    let nw = tl[grid_bounds.index(pos + V2::new(0, 0))];
                    let ne = tl[grid_bounds.index(pos + V2::new(1, 0))];
                    let sw = tl[grid_bounds.index(pos + V2::new(0, 1))];
                    let se = tl[grid_bounds.index(pos + V2::new(1, 1))];
                    let spec = TileSpec::from_corners([nw, ne, se, sw]);
                    specs[layer as usize][bounds.index(pos)] = spec;

                    let name = spec.name();
                    let z0_id =
                        if !spec.has_variants() {
                            get_id!(&name)
                        } else {
                            let rate = spec.variant_zero_rate();
                            // Give 1/rate chance of choosing any nonzero variant.
                            let v = rng.gen_range(0, 3 * rate);
                            // 1, 2, 3 map to themselves (variants).  Everything else maps to 0.
                            let v = if v >= 4 { 0 } else { v };
                            get_id!(&format!("{}/v{}", name, v))
                        };
                    gc.set_block(pos.extend(z + 0), z0_id);
                    if spec.has_cave() {
                        let z1_name = spec.cave_z1_name();
                        gc.set_block(pos.extend(z + 1), get_id!(&z1_name));
                    }
                }
            }
        }

        // Apply ramps
        let rel_bounds = Region::new(V2::new(-1, -1), V2::new(2, 2));
        let collect_bounds = Region::new(base + bounds.min - rel_bounds.max + scalar(1),
                                         base + bounds.max - rel_bounds.min);
        for r in &self.ctx.collect_points::<CaveRampsPass>(pid, collect_bounds) {
            let ramp_pos = r.pos - base;
            let z = r.layer as i32 * 2;

            let check = |x,y| {
                let p = ramp_pos + V2::new(x, y);
                if bounds.contains(p) {
                    Some(p)
                } else {
                    None
                }
            };

            if let Some(p) = check( -1, -1) {
                // Back left
                let spec = specs[r.layer as usize][bounds.index(p)];
                let name = spec.ramp_z1_name(0, 0);
                gc.set_block(p.extend(z + 1), get_id!(&name));
                let name = spec.ramp_z0_name(0, 0);
                gc.set_block(p.extend(z), get_id!(&name));
            }

            if let Some(p) = check(  1, -1) {
                // Back right
                let spec = specs[r.layer as usize][bounds.index(p)];
                let name = spec.ramp_z1_name(2, 0);
                gc.set_block(p.extend(z + 1), get_id!(&name));
                let name = spec.ramp_z0_name(2, 0);
                gc.set_block(p.extend(z), get_id!(&name));
            }

            if let Some(p) = check( -1,  0) {
                // Front left
                let spec = specs[r.layer as usize][bounds.index(p)];
                let name = spec.ramp_z1_name(0, 1);
                gc.set_block(p.extend(z + 1), get_id!(&name));
                let name = spec.ramp_z0_name(0, 1);
                gc.set_block(p.extend(z), get_id!(&name));
            }

            if let Some(p) = check(  1,  0) {
                // Front right
                let spec = specs[r.layer as usize][bounds.index(p)];
                let name = spec.ramp_z1_name(2, 1);
                gc.set_block(p.extend(z + 1), get_id!(&name));
                let name = spec.ramp_z0_name(2, 1);
                gc.set_block(p.extend(z), get_id!(&name));
            }

            if let Some(p) = check( 0, -1) {
                // Back center
                let spec = specs[r.layer as usize][bounds.index(p)];
                let name = spec.ramp_z1_name(1, 0);
                gc.set_block(p.extend(z + 1), get_id!(&name));
                let name = spec.ramp_z0_name(1, 0);
                gc.set_block(p.extend(z), get_id!(&name));

                let top_spec = specs[r.layer as usize + 1][bounds.index(p)];
                let terrain = match top_spec.terrain[3] {
                    'g' => "grass",
                    'c' => "dirt2",
                    _ => "dirt",
                };
                let name = format!("ramp/{}/cap0", terrain);
                gc.set_block(p.extend(z + 2), get_id!(&name));
            }

            if let Some(p) = check( 0,  0) {
                // Front center
                gc.set_block(p.extend(z + 1), get_id!("ramp/dirt2/z1"));

                let top_spec = specs[r.layer as usize + 1][bounds.index(p)];
                let terrain = match top_spec.terrain[0] {
                    'g' => "grass",
                    'c' => "dirt2",
                    _ => "dirt",
                };
                let name = format!("ramp/{}/cap1", terrain);
                gc.set_block(p.extend(z + 2), get_id!(&name));
            }

            if let Some(p) = check( 0,  1) {
                // Ramp bottom
                gc.set_block(p.extend(z + 0), get_id!("ramp/dirt2/z0"));
            }
        }

        macro_rules! template_id {
            ($name:expr) => (self.data.structure_templates.get_id($name))
        };

        // Apply cave junk
        let chest_id = template_id!("chest");
        for layer in 0 .. CHUNK_SIZE as u8 / 2 {
            let points = self.ctx.collect_points_layer::<CaveJunkPass>(pid, bounds + base, layer);
            for &pos in &points {
                let pos = pos - base;
                let spec = specs[layer as usize][bounds.index(pos)];
                if !spec.is_cave_floor() {
                    continue;
                }
                let pos = pos.extend(layer as i32 * 2);

                let opt_id = self.loot_tables.eval_structure_table(&mut rng, "cave/floor");
                if let Some(id) = opt_id {
                    let mut gs = GenStructure::new(pos, id);
                    if id == chest_id {
                        let contents = self.loot_tables.eval_item_table(&mut rng,
                                                                        "cave/chest");
                        let mut s = String::new();
                        for (item_id, count) in contents {
                            s.push_str(&format!("{}:{},",
                                                self.data.item_data.name(item_id),
                                                count));
                        }
                        gs.extra.insert("loot".to_owned(), s);
                    }
                    gc.structures.push(gs);
                }
            }
        }

        // Apply trees
        for t in &self.ctx.collect_points::<TreesPass>(pid, bounds + base) {
            let pos = (t.pos - base).extend(t.layer as i32 * 2);

            let opt_id = self.loot_tables.eval_structure_table(&mut rng, "forest/floor");
            if let Some(id) = opt_id {
                let gs = GenStructure::new(pos, id);
                gc.structures.push(gs);
            }
        }

        // Workbench (at spawn)
        if cpos == scalar(0) {
            let gs = GenStructure::new(scalar(0), template_id!("workbench"));
            gc.structures.push(gs);
        }

        gc
    }
}

#[derive(Clone, Copy, Debug)]
struct TileSpec {
    terrain: [char; 4],
    empty: [bool; 4],
    cave: [u8; 4],
}

impl TileSpec {
    fn empty() -> TileSpec {
        TileSpec {
            terrain: ['g'; 4],
            empty: [true; 4],
            cave: [1; 4],
        }
    }

    fn from_corners(corners: [Cell; 4]) -> TileSpec {
        let mut s = TileSpec::empty();

        for i in 0..4 {
            let c = &corners[i];

            s.terrain[i] = match c.floor_type {
                FloorType::Grass => 'g',
                FloorType::Cave => 'c',
                FloorType::Mountain => 'm',
                FloorType::Snow => 's',
                FloorType::Ash => 'a',
                FloorType::Water => 'w',
                FloorType::Lava => 'l',
                FloorType::Pit => 'p',
            };

            s.empty[i] = !c.flags.contains(terrain_grid::T_FLOOR);

            s.cave[i] =
                if c.flags.contains(terrain_grid::T_CAVE_INSIDE) { 2 }
                else if c.flags.contains(terrain_grid::T_CAVE) { 0 }
                else { 1 };
        }

        s
    }

    fn has_variants(&self) -> bool {
        !self.has_empty() &&
        !self.has_cave() &&
        self.terrain.iter().all(|&x| x == self.terrain[0])
    }

    fn variant_zero_rate(&self) -> i32 {
        match self.terrain[0] {
            'w' | 'l' => 100,
            'g' => 2,
            _ => 10,
        }
    }

    fn has_empty(&self) -> bool {
        self.empty != [false; 4]
    }

    fn has_cave(&self) -> bool {
        self.cave != [1; 4] &&
        self.cave != [2; 4]
    }

    fn is_cave_floor(&self) -> bool {
        self.empty != [true; 4] &&
        self.cave == [2; 4]
    }

    fn push_terrain_code(&self, s: &mut String) {
        for i in 0 .. 4 {
            s.push(self.terrain[i]);
        }
    }

    fn push_empty_code(&self, s: &mut String) {
        for i in 0 .. 4 {
            s.push(if self.empty[i] { '1' } else { '0' });
        }
    }

    fn push_cave_code(&self, s: &mut String) {
        for i in 0 .. 4 {
            s.push(match self.cave[i] {
               0 => '0',
               1 => '1',
               2 => '2',
               _ => unreachable!(),
            });
        }
    }

    fn name(&self) -> String {
        let mut s = String::new();
        s.push_str("terrain/");

        self.push_terrain_code(&mut s);

        if self.has_empty() {
            s.push_str("/e");
            self.push_empty_code(&mut s);
        }

        if self.has_cave() {
            s.push_str("/c");
            self.push_cave_code(&mut s);
        }

        s
    }

    fn cave_z1_name(&self) -> String {
        let mut s = String::new();
        s.push_str("cave_z1/c");
        self.push_cave_code(&mut s);
        s
    }

    fn ramp_z0_name(&self, x: i32, y: i32) -> String {
        let mut s = format!("ramp/xy{}{}/z0/", x, y);
        self.push_terrain_code(&mut s);
        s.push_str("/c");
        self.push_cave_code(&mut s);
        s
    }

    fn ramp_z1_name(&self, x: i32, y: i32) -> String {
        let mut s = format!("ramp/xy{}{}/z1/c", x, y);
        self.push_cave_code(&mut s);
        s
    }
}
