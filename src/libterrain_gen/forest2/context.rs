use std::fs::File;
use std::io;
use rand::{Rng, XorShiftRng, SeedableRng};

use libphysics::CHUNK_SIZE;
use libserver_config::Storage;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};

use cache::{Cache, Summary};

use forest2::height_map::{self, HeightMap};
use forest2::height_detail::{self, HeightDetail};
use forest2::cave_ramps::{self, RampPositions, CaveRamps};
use forest2::cave_detail::{self, CaveDetail};
use forest2::cave_junk::{self, CaveJunk};
use forest2::trees::{self, TreePositions, Trees};
use forest2::terrain_grid::{self, TerrainGrid};


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

pub struct Context<'d> {
    rng: XorShiftRng,
    globals: Cache<'d, PlaneGlobals>,
    height_map: Cache<'d, HeightMap>,
    height_detail: Cache<'d, HeightDetail>,
    cave_ramp_positions: Cache<'d, RampPositions>,
    cave_ramps: Cache<'d, CaveRamps>,
    cave_detail: Cache<'d, CaveDetail>,
    cave_junk: Cache<'d, CaveJunk>,
    tree_positions: Cache<'d, TreePositions>,
    trees: Cache<'d, Trees>,
    terrain_grid: Cache<'d, TerrainGrid>,
}

impl<'d> Context<'d> {
    pub fn new(storage: &'d Storage, rng: XorShiftRng) -> Context<'d> {
        Context {
            rng: rng,
            globals: Cache::new(storage, "globals"),
            height_map: Cache::new(storage, "height_map"),
            height_detail: Cache::new(storage, "height_detail"),
            cave_ramp_positions: Cache::new(storage, "cave_ramp_positions"),
            cave_ramps: Cache::new(storage, "cave_ramps"),
            cave_detail: Cache::new(storage, "cave_detail"),
            cave_junk: Cache::new(storage, "cave_junk"),
            tree_positions: Cache::new(storage, "tree_positions"),
            trees: Cache::new(storage, "trees"),
            terrain_grid: Cache::new(storage, "terrain_grid"),
        }
    }


    fn globals_mut(&mut self, pid: Stable<PlaneId>) -> &mut PlaneGlobals {
        if let Ok(()) = self.globals.load(pid, scalar(0)) {
            self.globals.get_mut(pid, scalar(0))
        } else {
            let g = self.globals.create(pid, scalar(0));
            *g = PlaneGlobals::new(&mut self.rng);
            g
        }
    }

    pub fn globals(&mut self, pid: Stable<PlaneId>) -> &PlaneGlobals {
        self.globals_mut(pid)
    }

    pub fn get_rng(&mut self, pid: Stable<PlaneId>) -> XorShiftRng {
        self.globals_mut(pid).rng.gen()
    }


    #[inline]
    fn entry<T, F, G>(&mut self,
                      pid: Stable<PlaneId>,
                      pos: V2,
                      get_cache: F,
                      generate: G) -> &T
            where T: Summary,
                  F: for<'a> Fn(&'a mut Context<'d>) -> &'a mut Cache<'d, T>,
                  G: FnOnce(&mut Context, &mut T, Stable<PlaneId>, V2) {
        if let Ok(()) = get_cache(self).load(pid, pos) {
            get_cache(self).get(pid, pos)
        } else {
            let mut x = T::alloc();
            generate(self, &mut *x, pid, pos);
            get_cache(self).insert(pid, pos, x)
        }
    }

    #[inline]
    fn get_entry<T, F>(&mut self,
                       pid: Stable<PlaneId>,
                       pos: V2,
                       get_cache: F) -> Option<&T>
            where T: Summary,
                  F: for<'a> Fn(&'a mut Context<'d>) -> &'a mut Cache<'d, T> {
        if let Ok(()) = get_cache(self).load(pid, pos) {
            Some(get_cache(self).get(pid, pos))
        } else {
            None
        }
    }


    pub fn height_map(&mut self, pid: Stable<PlaneId>, pos: V2) -> &HeightMap {
        self.entry(pid, pos,
                   |ctx| &mut ctx.height_map,
                   height_map::generate)
    }

    pub fn point_height(&mut self, pid: Stable<PlaneId>, pos: V2) -> i32 {
        let size = scalar(height_map::HEIGHTMAP_SIZE as i32);
        let cpos = pos.div_floor(size);
        let bounds = Region::new(cpos * size, (cpos + scalar(1)) * size);
        let hm = self.height_map(pid, cpos);
        hm.buf[bounds.index(pos)]
    }

    pub fn height_detail(&mut self, pid: Stable<PlaneId>, pos: V2) -> &HeightDetail {
        self.entry(pid, pos,
                   |ctx| &mut ctx.height_detail,
                   height_detail::generate)
    }

    pub fn point_height_detail(&mut self, pid: Stable<PlaneId>, pos: V2) -> i32 {
        let size = scalar(CHUNK_SIZE);
        let cpos = pos.div_floor(size);
        let bounds = Region::new(cpos * size, (cpos + scalar(1)) * size);
        let hm = self.height_detail(pid, cpos);
        hm.buf[bounds.index(pos)]
    }

    pub fn cave_ramps(&mut self,
                      pid: Stable<PlaneId>,
                      pos: V2) -> &CaveRamps {
        self.entry(pid, pos,
                   |ctx| &mut ctx.cave_ramps,
                   cave_ramps::generate)
    }

    pub fn get_cave_ramps(&mut self,
                          pid: Stable<PlaneId>,
                          pos: V2) -> Option<&CaveRamps> {
        self.get_entry(pid, pos,
                       |ctx| &mut ctx.cave_ramps)
    }

    pub fn cave_ramp_positions(&mut self,
                               pid: Stable<PlaneId>,
                               pos: V2) -> &RampPositions {
        self.entry(pid, pos,
                   |ctx| &mut ctx.cave_ramp_positions,
                   cave_ramps::generate_positions)
    }

    pub fn get_cave_ramp_positions(&mut self,
                                   pid: Stable<PlaneId>,
                                   pos: V2) -> Option<&RampPositions> {
        self.get_entry(pid, pos,
                       |ctx| &mut ctx.cave_ramp_positions)
    }

    pub fn cave_detail(&mut self, pid: Stable<PlaneId>, pos: V2) -> &CaveDetail {
        self.entry(pid, pos,
                   |ctx| &mut ctx.cave_detail,
                   cave_detail::generate)
    }

    pub fn get_cave_detail(&mut self, pid: Stable<PlaneId>, pos: V2) -> Option<&CaveDetail> {
        self.get_entry(pid, pos,
                       |ctx| &mut ctx.cave_detail)
    }

    pub fn cave_junk(&mut self, pid: Stable<PlaneId>, pos: V2) -> &CaveJunk {
        self.entry(pid, pos,
                   |ctx| &mut ctx.cave_junk,
                   cave_junk::generate)
    }

    pub fn get_cave_junk(&mut self, pid: Stable<PlaneId>, pos: V2) -> Option<&CaveJunk> {
        self.get_entry(pid, pos,
                       |ctx| &mut ctx.cave_junk)
    }

    pub fn tree_positions(&mut self, pid: Stable<PlaneId>, pos: V2) -> &TreePositions {
        self.entry(pid, pos,
                   |ctx| &mut ctx.tree_positions,
                   trees::generate_positions)
    }

    pub fn get_tree_positions(&mut self, pid: Stable<PlaneId>, pos: V2) -> Option<&TreePositions> {
        self.get_entry(pid, pos,
                       |ctx| &mut ctx.tree_positions)
    }

    pub fn trees(&mut self, pid: Stable<PlaneId>, pos: V2) -> &Trees {
        self.entry(pid, pos,
                   |ctx| &mut ctx.trees,
                   trees::generate)
    }

    pub fn terrain_grid(&mut self, pid: Stable<PlaneId>, pos: V2) -> &TerrainGrid {
        self.entry(pid, pos,
                   |ctx| &mut ctx.terrain_grid,
                   terrain_grid::generate)
    }
}
