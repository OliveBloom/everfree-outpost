use std::fs::File;
use std::io;
use std::mem;
use rand::{Rng, XorShiftRng, SeedableRng};

use libphysics::CHUNK_SIZE;
use libserver_config::Storage;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};

use cache::{Cache, Summary};
use forest2;


pub struct PlaneGlobals {
    pub rng: XorShiftRng,
    pub heightmap_seed: u64,
}

impl PlaneGlobals {
    fn new<R: Rng>(rng: &mut R) -> PlaneGlobals {
        PlaneGlobals {
            rng: rng.gen(),
            heightmap_seed: rng.gen(),
        }
    }
}

impl Summary for PlaneGlobals {
    fn alloc() -> Box<PlaneGlobals> {
        Box::new(PlaneGlobals {
            rng: XorShiftRng::new_unseeded(),
            heightmap_seed: 0,
        })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        let mut rng = self.rng.clone();
        let rng_seed: (u32, u32, u32, u32) = rng.gen();
        try!(f.write_bytes(rng_seed));

        try!(f.write_bytes(self.heightmap_seed));
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<PlaneGlobals>> {
        let rng_seed: (u32, u32, u32, u32) = try!(f.read_bytes());
        let rng = XorShiftRng::from_seed([rng_seed.0, rng_seed.1, rng_seed.2, rng_seed.3]);

        let heightmap_seed = try!(f.read_bytes());

        Ok(Box::new(PlaneGlobals {
            rng: rng,
            heightmap_seed: heightmap_seed,
        }))
    }
}


pub const HEIGHTMAP_SIZE: usize = 64;
pub struct HeightMap {
    pub buf: [i32; HEIGHTMAP_SIZE * HEIGHTMAP_SIZE],
}

impl Summary for HeightMap {
    fn alloc() -> Box<HeightMap> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        f.write_bytes_slice(&self.buf)
    }

    fn read_from(mut f: File) -> io::Result<Box<HeightMap>> {
        let mut result = HeightMap::alloc();
        try!(f.read_bytes_slice(&mut result.buf));
        Ok(result)
    }
}


pub type CaveDetailLayer = [u8; (CHUNK_SIZE * CHUNK_SIZE) as usize / 8];
pub struct CaveDetail {
    pub buf: [CaveDetailLayer; CHUNK_SIZE as usize / 2],
}

impl Summary for CaveDetail {
    fn alloc() -> Box<CaveDetail> {
        Box::new(unsafe { mem::zeroed() })
    }

    fn write_to(&self, mut f: File) -> io::Result<()> {
        for layer in &self.buf {
            try!(f.write_bytes_slice(layer));
        }
        Ok(())
    }

    fn read_from(mut f: File) -> io::Result<Box<CaveDetail>> {
        let mut result = CaveDetail::alloc();
        for layer in &mut result.buf {
            try!(f.read_bytes_slice(layer));
        }
        Ok(result)
    }
}


bitflags! {
    flags TerrainFlags: u8 {
        const T_FLOOR       = 0x01,
        const T_WALL        = 0x02,
        const T_WATER       = 0x04,
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



pub struct Context<'d> {
    rng: XorShiftRng,
    globals: Cache<'d, PlaneGlobals>,
    height_map: Cache<'d, HeightMap>,
    cave_detail: Cache<'d, CaveDetail>,
    terrain_grid: Cache<'d, TerrainGrid>,
}

impl<'d> Context<'d> {
    pub fn new(storage: &'d Storage, rng: XorShiftRng) -> Context<'d> {
        Context {
            rng: rng,
            globals: Cache::new(storage, "globals"),
            height_map: Cache::new(storage, "height_map"),
            cave_detail: Cache::new(storage, "cave_detail"),
            terrain_grid: Cache::new(storage, "terrain_grid"),
        }
    }

    pub fn globals(&mut self, pid: Stable<PlaneId>) -> &PlaneGlobals {
        if let Ok(()) = self.globals.load(pid, scalar(0)) {
            self.globals.get(pid, scalar(0))
        } else {
            let g = self.globals.create(pid, scalar(0));
            *g = PlaneGlobals::new(&mut self.rng);
            g
        }
    }

    pub fn height_map(&mut self, pid: Stable<PlaneId>, pos: V2) -> &HeightMap {
        if let Ok(()) = self.height_map.load(pid, pos) {
            self.height_map.get(pid, pos)
        } else {
            let mut x = HeightMap::alloc();
            forest2::height_map::generate(self, &mut *x, pid, pos);
            self.height_map.insert(pid, pos, x)
        }
    }

    pub fn point_height(&mut self, pid: Stable<PlaneId>, pos: V2) -> i32 {
        let size = scalar(HEIGHTMAP_SIZE as i32);
        let cpos = pos.div_floor(size);
        let bounds = Region::new(cpos * size, (cpos + scalar(1)) * size);
        let hm = self.height_map(pid, cpos);
        hm.buf[bounds.index(pos)]
    }

    pub fn cave_detail(&mut self, pid: Stable<PlaneId>, pos: V2) -> &CaveDetail {
        if let Ok(()) = self.cave_detail.load(pid, pos) {
            self.cave_detail.get(pid, pos)
        } else {
            let mut x = CaveDetail::alloc();
            //forest2::cave_detail::generate(self, &mut *x, pid, pos);
            self.cave_detail.insert(pid, pos, x)
        }
    }

    pub fn get_cave_detail(&mut self, pid: Stable<PlaneId>, pos: V2) -> Option<&CaveDetail> {
        if let Ok(()) = self.cave_detail.load(pid, pos) {
            Some(self.cave_detail.get(pid, pos))
        } else {
            None
        }
    }

    pub fn terrain_grid(&mut self, pid: Stable<PlaneId>, pos: V2) -> &TerrainGrid {
        if let Ok(()) = self.terrain_grid.load(pid, pos) {
            self.terrain_grid.get(pid, pos)
        } else {
            let mut x = TerrainGrid::alloc();
            forest2::terrain_grid::generate(self, &mut *x, pid, pos);
            self.terrain_grid.insert(pid, pos, x)
        }
    }
}
