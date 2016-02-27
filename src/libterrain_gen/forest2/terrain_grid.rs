use std::cmp;
use std::fs::File;
use std::io;
use std::mem;

use libphysics::CHUNK_SIZE;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};

use cache::Summary;
use forest2::context::Context;
use forest2::cave_ramps;


bitflags! {
    flags TerrainFlags: u8 {
        const T_FLOOR       = 0x01,
        const T_CAVE        = 0x02,
        const T_CAVE_INSIDE = 0x04,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum FloorType {
    Grass = 0,      // So we can init with mem::zeroed()
    Cave = 1,
    Mountain = 2,
    Snow = 3,
    Ash = 4,
    Water = 5,
    Lava = 6,
    Pit = 7,
}

impl FloorType {
    fn from_primitive(x: u8) -> Option<FloorType> {
        use self::FloorType::*;
        match x {
            0 => Some(Grass),
            1 => Some(Cave),
            2 => Some(Mountain),
            3 => Some(Snow),
            4 => Some(Ash),
            5 => Some(Water),
            6 => Some(Lava),
            7 => Some(Pit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub flags: TerrainFlags,
    pub floor_type: FloorType,
}
pub type TerrainLayer = [Cell; ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) as usize];
/// Full terrain information for each grid intersection in a chunk.  This is the final
/// representation used to generate the chunk's block data.
pub struct TerrainGrid {
    pub buf: [TerrainLayer; (CHUNK_SIZE / 2) as usize],
}

impl Summary for TerrainGrid {
    fn alloc() -> Box<TerrainGrid> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        let mut buffer = [(0, 0); ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) as usize];
        for layer in 0 .. (CHUNK_SIZE / 2) as usize {
            for i in 0 .. buffer.len() {
                let info = &self.buf[layer][i];
                buffer[i] = (info.flags.bits(), info.floor_type as u8);
            }
            try!(f.write_bytes_slice(&buffer));
        }
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<TerrainGrid>> {
        let mut result = TerrainGrid::alloc();
        let mut buffer = [(0, 0); ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) as usize];
        for layer in 0 .. (CHUNK_SIZE / 2) as usize {
            try!(f.read_bytes_slice(&mut buffer));
            for i in 0 .. buffer.len() {
                let (raw_flags, raw_floor_type) = buffer[i];
                let flags = match TerrainFlags::from_bits(raw_flags) {
                    Some(x) => x,
                    None => {
                        return Err(io::Error::new(io::ErrorKind::Other,
                                                  "invalid terran flags"));
                    },
                };
                let floor_type = match FloorType::from_primitive(raw_floor_type) {
                    Some(x) => x,
                    None => {
                        return Err(io::Error::new(io::ErrorKind::Other,
                                                  "invalid floor type"));
                    },
                };

                result.buf[layer][i].flags = flags;
                result.buf[layer][i].floor_type = floor_type;
            }
        }
        Ok(result)
    }
}


pub fn generate(ctx: &mut Context,
                chunk: &mut TerrainGrid,
                pid: Stable<PlaneId>,
                cpos: V2) {
    let bounds = Region::<V2>::new(scalar(0), scalar(CHUNK_SIZE + 1));

    // Apply heightmap info
    {
        let detail = ctx.height_detail(pid, cpos);
        for p in bounds.points() {
            let h = detail.buf[bounds.index(p)];
            let h =
                if h.abs() >= 256 {
                    //warn!("perlin noise value exceeds bounds: {} @ {:?}", h, p);
                    255 * h.signum()
                } else {
                    h
                };
            let block_height = cmp::max(0, h / 32) as usize;
            let idx = bounds.index(p);
            for layer in 0 .. block_height {
                chunk.buf[layer][idx].flags.insert(T_CAVE | T_FLOOR);
                chunk.buf[layer][idx].floor_type = FloorType::Cave;
            }
            chunk.buf[block_height][idx].flags.insert(T_FLOOR);

            if h < -128 {
                chunk.buf[0][idx].floor_type = FloorType::Water;
            }
        }
    }

    // Apply cave info
    {
        let detail = ctx.cave_detail(pid, cpos);
        for layer in 0 .. CHUNK_SIZE as usize / 2 {
            for p in bounds.points() {
                if detail.layer(layer).get(bounds.index(p)) {
                    // It's a wall.
                    continue;
                }

                if chunk.buf[layer][bounds.index(p)].flags.contains(T_CAVE) {
                    chunk.buf[layer][bounds.index(p)].flags.insert(T_CAVE_INSIDE);
                }
            }
        }
    }

    // HACK
    let bounds_global = bounds + cpos * scalar(CHUNK_SIZE);
    for r in &cave_ramps::ramps_in_region(ctx, pid, bounds_global) {
        info!("  applying ramp @ {:?} z={}", r.pos, r.layer);
        for p in Region::new(r.pos, r.pos + V2::new(2, 3)).intersect(bounds_global).points() {
            for z in 0 .. 8 {
                chunk.buf[z][bounds_global.index(p)].floor_type = FloorType::Cave;
            }
        }
    }
}
