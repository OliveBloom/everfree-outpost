use chunks::{self, Chunks};
use engine::split::{EngineRef, Open, Part};
use terrain_gen;
use vision::{self, Vision};
use world::{self, World};


macro_rules! parts {
    ($($name:ident),* ,) => { parts!($($name),*) };
    ($($name:ident),*) => {
        $( engine_part_typedef!(pub $name); )*
    }
}


macro_rules! impl_slice {
    ($($from:ident :: $method:ident -> $to:ident;)*) => {
        $(
            impl<'a, 'd> $from<'a, 'd> {
                pub fn $method<'b>(&'b mut self) -> $to<'b, 'd> {
                    $to(self.borrow().0.slice())
                }
            }
        )*
    };
}


parts!(WorldFragment, WorldHooks, VisionFragment, VisionHooks);

impl<'a, 'd> world::Fragment<'d> for WorldFragment<'a, 'd> {
    fn world(&self) -> &World<'d> {
        (**self).world()
    }

    fn world_mut(&mut self) -> &mut World<'d> {
        (**self).world_mut()
    }

    type H = WorldHooks<'a, 'd>;
    fn with_hooks<F, R>(&mut self, f: F) -> R
            where F: FnOnce(&mut WorldHooks<'a, 'd>) -> R {
        let e = unsafe { self.borrow().fiddle().to_part().slice() };
        f(&mut Part::from_part(e))
    }
}

impl_slice! {
    EngineRef::as_world_fragment -> WorldFragment;
    EngineRef::as_vision_fragment -> VisionFragment;
    WorldHooks::as_vision_fragment -> VisionFragment;
    WorldHooks::as_hidden_world_fragment -> HiddenWorldFragment;
}


parts!(HiddenWorldFragment, HiddenWorldHooks, HiddenVisionFragment);

impl<'a, 'd> world::Fragment<'d> for HiddenWorldFragment<'a, 'd> {
    fn world(&self) -> &World<'d> {
        (**self).world()
    }

    fn world_mut(&mut self) -> &mut World<'d> {
        (**self).world_mut()
    }

    type H = HiddenWorldHooks<'a, 'd>;
    fn with_hooks<F, R>(&mut self, f: F) -> R
            where F: FnOnce(&mut HiddenWorldHooks<'a, 'd>) -> R {
        let e = unsafe { self.borrow().fiddle().to_part().slice() };
        f(&mut Part::from_part(e))
    }
}

impl_slice! {
    EngineRef::as_hidden_world_fragment -> HiddenWorldFragment;
    EngineRef::as_hidden_vision_fragment -> HiddenVisionFragment;
    HiddenWorldHooks::as_hidden_vision_fragment -> HiddenVisionFragment;
}


parts!(TerrainGenFragment);

impl<'a, 'd> terrain_gen::Fragment<'d> for TerrainGenFragment<'a, 'd> {
    fn terrain_gen_mut(&mut self) -> &mut terrain_gen::TerrainGen {
        (**self).terrain_gen_mut()
    }

    type WF = WorldFragment<'a, 'd>;
    fn with_world<F, R>(&mut self, f: F) -> R
            where F: FnOnce(&mut WorldFragment<'a, 'd>) -> R {
        let e = unsafe { self.borrow().fiddle().to_part().slice() };
        f(&mut Part::from_part(e))
    }
}

impl_slice! {
    EngineRef::as_terrain_gen_fragment -> TerrainGenFragment;
}
