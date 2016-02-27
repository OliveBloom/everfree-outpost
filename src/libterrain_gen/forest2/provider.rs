use rand::XorShiftRng;

use libphysics::CHUNK_SIZE;
use libserver_config::{Data, Storage};
use libserver_types::*;

use GenChunk;
use forest2::context::Context;
use forest2::terrain_grid::{self, Cell, FloorType};
use forest2::cave_ramps;

pub struct Provider<'d> {
    data: &'d Data,
    ctx: Context<'d>,
}

impl<'d> Provider<'d> {
    pub fn new(data: &'d Data, storage: &'d Storage, rng: XorShiftRng) -> Provider<'d> {
        Provider {
            data: data,
            ctx: Context::new(storage, rng),
        }
    }

    pub fn generate(&mut self, pid: Stable<PlaneId>, cpos: V2) -> GenChunk {
        let mut gc = GenChunk::new();

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

        let mut specs = Box::new(
            [[TileSpec::empty(); CHUNK_SIZE as usize * CHUNK_SIZE as usize]; 8]);

        {
            let t = self.ctx.terrain_grid(pid, cpos);

            // Apply terrain grid
            for layer in 0 .. CHUNK_SIZE / 2 {
                let z = layer * 2;
                let tl = &t.buf[layer as usize];
                for pos in bounds.points() {
                    let nw = tl[grid_bounds.index(pos + V2::new(0, 0))];
                    let ne = tl[grid_bounds.index(pos + V2::new(1, 0))];
                    let sw = tl[grid_bounds.index(pos + V2::new(0, 1))];
                    let se = tl[grid_bounds.index(pos + V2::new(1, 1))];
                    let spec = TileSpec::from_corners([nw, ne, se, sw]);
                    specs[layer as usize][bounds.index(pos)] = spec;

                    let name = spec.name();
                    gc.set_block(pos.extend(z + 0), get_id!(&name));
                    if spec.has_cave() {
                        let z1_name = spec.cave_z1_name();
                        gc.set_block(pos.extend(z + 1), get_id!(&z1_name));
                    }
                }
            }
        }

        // Apply ramps
        for r in &cave_ramps::ramps_in_region(&mut self.ctx, pid,
                                              bounds.expand(scalar(1)) + base) {
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

    fn has_empty(&self) -> bool {
        self.empty != [false; 4]
    }

    fn has_cave(&self) -> bool {
        self.cave != [1; 4] &&
        self.cave != [2; 4]
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
        s.push_str("cave_z1/");
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
