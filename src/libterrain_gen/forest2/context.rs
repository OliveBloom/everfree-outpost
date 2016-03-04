use std::fs::File;
use std::io;
use std::ops::Index;
use rand::{Rng, XorShiftRng, SeedableRng};

use libphysics::CHUNK_SIZE;
use libserver_config::Storage;
use libserver_types::*;
use libserver_util::bytes::{ReadBytes, WriteBytes};

use cache::{Cache, Summary};
use forest2::common::{self, GenPass, GridLike, PointsLike, HasPos};

use forest2::height_map::{self, HeightMap};
use forest2::height_detail::{self, HeightDetail};
use forest2::cave_ramps::{self, RampPositions, CaveRamps};
use forest2::cave_detail::{self, CaveDetail};
use forest2::cave_junk::{self, CaveJunk};
use forest2::trees::{self, TreePositions, Trees};
use forest2::terrain_grid::{self, TerrainGrid};


macro_rules! define_gen_pass {
    ($Pass:ident ( $Value:ty ): $name:ident) => {
        define_gen_pass!($Pass ( $Value ): $name, $name::generate);
    };
    ($Pass:ident ( $Value:ty ): $field:ident, $generate:path) => {
        pub struct $Pass;
        
        impl GenPass for $Pass {
            type Key = (Stable<PlaneId>, V2);
            type Value = $Value;

            fn field_mut<'a, 'd>(ctx: &'a mut Context<'d>)
                                 -> &'a mut Cache<'d, Self::Key, Self::Value> {
                &mut ctx.$field
            }

            fn generate(ctx: &mut Context, (pid, pos): Self::Key, value: &mut Self::Value) {
                $generate(ctx, value, pid, pos);
            }
        }
    }
}

macro_rules! define_gen_pass_layered {
    ($Pass:ident ( $Value:ty ): $name:ident) => {
        define_gen_pass_layered!($Pass ( $Value ): $name, $name::generate);
    };
    ($Pass:ident ( $Value:ty ): $field:ident, $generate:path) => {
        pub struct $Pass;

        impl GenPass for $Pass {
            type Key = (Stable<PlaneId>, V2, u8);
            type Value = $Value;

            fn field_mut<'a, 'd>(ctx: &'a mut Context<'d>)
                                 -> &'a mut Cache<'d, Self::Key, Self::Value> {
                &mut ctx.$field
            }

            fn generate(ctx: &mut Context, (pid, pos, layer): Self::Key, value: &mut Self::Value) {
                $generate(ctx, value, pid, pos, layer);
            }
        }
    };
}


pub struct PlaneGlobalsPass;

impl GenPass for PlaneGlobalsPass {
    type Key = Stable<PlaneId>;
    type Value = PlaneGlobals;

    fn field_mut<'a, 'd>(ctx: &'a mut Context<'d>)
                         -> &'a mut Cache<'d, Self::Key, Self::Value> {
        &mut ctx.globals
    }

    fn generate(ctx: &mut Context, key: Self::Key, value: &mut Self::Value) {
        *value = PlaneGlobals::new(&mut ctx.rng);
    }
}


define_gen_pass!(HeightMapPass(HeightMap): height_map);
define_gen_pass!(HeightDetailPass(HeightDetail): height_detail);
define_gen_pass_layered!(CaveDetailPass(CaveDetail): cave_detail);
define_gen_pass!(RampPositionsPass(RampPositions):
                 cave_ramp_positions, cave_ramps::generate_positions);
define_gen_pass!(CaveRampsPass(CaveRamps): cave_ramps);
define_gen_pass_layered!(CaveJunkPass(CaveJunk): cave_junk);
define_gen_pass!(TreePositionsPass(TreePositions):
                 tree_positions, trees::generate_positions);
define_gen_pass!(TreesPass(Trees): trees);
define_gen_pass!(TerrainGridPass(TerrainGrid): terrain_grid);


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
    globals: Cache<'d, Stable<PlaneId>, PlaneGlobals>,
    height_map: Cache<'d, (Stable<PlaneId>, V2), HeightMap>,
    height_detail: Cache<'d, (Stable<PlaneId>, V2), HeightDetail>,
    cave_ramp_positions: Cache<'d, (Stable<PlaneId>, V2), RampPositions>,
    cave_ramps: Cache<'d, (Stable<PlaneId>, V2), CaveRamps>,
    cave_detail: Cache<'d, (Stable<PlaneId>, V2, u8), CaveDetail>,
    cave_junk: Cache<'d, (Stable<PlaneId>, V2, u8), CaveJunk>,
    tree_positions: Cache<'d, (Stable<PlaneId>, V2), TreePositions>,
    trees: Cache<'d, (Stable<PlaneId>, V2), Trees>,
    terrain_grid: Cache<'d, (Stable<PlaneId>, V2), TerrainGrid>,
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
        if let Ok(()) = self.globals.load(pid) {
            self.globals.get_mut(pid)
        } else {
            let g = self.globals.create(pid);
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


    pub fn result<P: GenPass>(&mut self, key: P::Key) -> &P::Value {
        if let Ok(()) = P::field_mut(self).load(key) {
            P::field_mut(self).get(key)
        } else {
            let mut value = P::Value::alloc();
            P::generate(self, key, &mut value);
            P::field_mut(self).insert(key, value)
        }
    }

    pub fn result_mut<P: GenPass>(&mut self, key: P::Key) -> &mut P::Value {
        if let Ok(()) = P::field_mut(self).load(key) {
            P::field_mut(self).get_mut(key)
        } else {
            let mut value = P::Value::alloc();
            P::generate(self, key, &mut value);
            P::field_mut(self).insert(key, value)
        }
    }

    pub fn get_result<P: GenPass>(&mut self, key: P::Key) -> Option<&P::Value> {
        if let Ok(()) = P::field_mut(self).load(key) {
            Some(P::field_mut(self).get(key))
        } else {
            None
        }
    }


    pub fn grid_fold<P, F, S>(&mut self,
                              pid: Stable<PlaneId>,
                              bounds: Region<V2>,
                              init: S,
                              mut f: F) -> S
            where P: GenPass<Key=(Stable<PlaneId>, V2)>,
                  P::Value: GridLike,
                  F: FnMut(S, V2, <P::Value as GridLike>::Elem) -> S,
                  <P::Value as GridLike>::Elem: ::std::fmt::Display {
        let spacing = <P::Value as GridLike>::spacing();
        let size = <P::Value as GridLike>::size();
        let grid_bounds = bounds.div_round_signed(spacing);

        let mut state = init;
        for gpos in grid_bounds.points() {
            let chunk = self.result::<P>((pid, gpos));
            let base = gpos * scalar(spacing);
            let chunk_bounds = Region::new(base, base + scalar(size));
            for p in bounds.intersect(chunk_bounds).points() {
                let val = chunk.get(p - base);
                state = f(state, p, val);
            }
        }

        state
    }

    pub fn points_fold<P, F, S>(&mut self,
                                pid: Stable<PlaneId>,
                                bounds: Region<V2>,
                                init: S,
                                mut f: F) -> S
            where P: GenPass<Key=(Stable<PlaneId>, V2)>,
                  P::Value: PointsLike,
                  F: FnMut(S, V2, &<P::Value as PointsLike>::Elem) -> S {
        let spacing = <P::Value as PointsLike>::spacing();
        let grid_bounds = bounds.div_round_signed(spacing);

        let mut state = init;
        for gpos in grid_bounds.points() {
            let chunk = self.result::<P>((pid, gpos));
            let base = gpos * scalar(spacing);
            for val in chunk.as_slice() {
                let abs_pos = val.pos() + base;
                if bounds.contains(abs_pos) {
                    state = f(state, abs_pos, val);
                }
            }
        }

        state
    }

    pub fn collect_points<P>(&mut self,
                             pid: Stable<PlaneId>,
                             bounds: Region<V2>) -> Vec<<P::Value as PointsLike>::Elem>
            where P: GenPass<Key=(Stable<PlaneId>, V2)>,
                  P::Value: PointsLike {
        let mut v = Vec::new();
        self.points_fold::<P, _, ()>(pid, bounds, (), |(), pos, val| {
            v.push(val.with_pos(pos));
        });
        v
    }

    pub fn points_fold_layer<P, F, S>(&mut self,
                                      pid: Stable<PlaneId>,
                                      bounds: Region<V2>,
                                      layer: u8,
                                      init: S,
                                      mut f: F) -> S
            where P: GenPass<Key=(Stable<PlaneId>, V2, u8)>,
                  P::Value: PointsLike,
                  F: FnMut(S, V2, &<P::Value as PointsLike>::Elem) -> S {
        let spacing = <P::Value as PointsLike>::spacing();
        let grid_bounds = bounds.div_round_signed(spacing);

        let mut state = init;
        for gpos in grid_bounds.points() {
            let chunk = self.result::<P>((pid, gpos, layer));
            let base = gpos * scalar(spacing);
            for val in chunk.as_slice() {
                let abs_pos = val.pos() + base;
                if bounds.contains(abs_pos) {
                    state = f(state, abs_pos, val);
                }
            }
        }

        state
    }

    pub fn collect_points_layer<P>(&mut self,
                                   pid: Stable<PlaneId>,
                                   bounds: Region<V2>,
                                   layer: u8) -> Vec<<P::Value as PointsLike>::Elem>
            where P: GenPass<Key=(Stable<PlaneId>, V2, u8)>,
                  P::Value: PointsLike {
        let mut v = Vec::new();
        self.points_fold_layer::<P, _, ()>(pid, bounds, layer, (), |(), pos, val| {
            v.push(val.with_pos(pos));
        });
        v
    }
}
