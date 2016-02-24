use rand::XorShiftRng;

use libphysics::CHUNK_SIZE;
use libserver_config::{Data, Storage};
use libserver_types::*;

use {GenChunk, GenStructure};
use forest2::context::{self, Context, Cell, FloorType};

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
        let t = self.ctx.terrain_grid(pid, cpos);

        let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE));
        let grid_bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));

        for layer in 0 .. CHUNK_SIZE / 2 - 4{
            let z = layer * 2;
            let tl = &t.buf[layer as usize];
            for pos in bounds.points() {
                let nw = tl[grid_bounds.index(pos + V2::new(0, 0))];
                let ne = tl[grid_bounds.index(pos + V2::new(1, 0))];
                let sw = tl[grid_bounds.index(pos + V2::new(0, 1))];
                let se = tl[grid_bounds.index(pos + V2::new(1, 1))];
                let (name, z1_name) = calc_tile_name([nw, ne, se, sw]);

                let z0_id = match self.data.block_data.find_id(&name) {
                    Some(x) => x,
                    None => { warn!("no such block: {}", name); 0 }
                };
                gc.set_block(pos.extend(z + 0), z0_id);
                if let Some(z1_name) = z1_name {
                    let z1_id = match self.data.block_data.find_id(&z1_name) {
                        Some(x) => x,
                        None => { warn!("no such block: {}", z1_name); 0 }
                    };
                    gc.set_block(pos.extend(z + 1), z1_id);
                }
            }
        }

        gc
    }
}

fn calc_tile_name(corners: [Cell; 4]) -> (String, Option<String>) {
    let mut s = String::new();
    let mut opt_z1 = None;

    s.push_str("terrain/");
    for c in &corners {
        let code = match c.floor_type {
            FloorType::Grass => "g",
            FloorType::Cave => "c",
            FloorType::Mountain => "m",
            FloorType::Snow => "s",
            FloorType::Ash => "a",
            FloorType::Water => "w",
            FloorType::Lava => "l",
            FloorType::Pit => "p",
        };
        s.push_str(code);
    }

    if corners.iter().any(|c| !c.flags.contains(context::T_FLOOR)) {
        s.push_str("/e");
        for c in &corners {
            s.push_str(if c.flags.contains(context::T_FLOOR) { "0" } else { "1" });
        }
    }

    let has_cave = corners.iter().any(|c| c.flags.contains(context::T_CAVE));
    if has_cave {
        let mut z1 = String::new();
        s.push_str("/c");
        z1.push_str("cave_z1/");
        for c in &corners {
            // 0 - cave (blocked), 1 - outside, 2 - cave interior
            let code =
                if c.flags.contains(context::T_CAVE_INSIDE) { "2" }
                else if c.flags.contains(context::T_CAVE) { "0" }
                else { "1" };
            s.push_str(code);
            z1.push_str(code);
        }

        opt_z1 = Some(z1);
    }

    (s, opt_z1)
}
