use engine::split::{EngineRef, Part};
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


parts!(WorldFragment);

impl<'a, 'd> world::Fragment<'d> for WorldFragment<'a, 'd> {
    fn world(&self) -> &World<'d> {
        (**self).world()
    }

    fn world_mut(&mut self) -> &mut World<'d> {
        (**self).world_mut()
    }
}

impl_slice! {
    EngineRef::as_world_fragment -> WorldFragment;
}


parts!(HiddenWorldFragment);

impl<'a, 'd> world::Fragment<'d> for HiddenWorldFragment<'a, 'd> {
    fn world(&self) -> &World<'d> {
        (**self).world()
    }

    fn world_mut(&mut self) -> &mut World<'d> {
        (**self).world_mut()
    }
}

impl_slice! {
    EngineRef::as_hidden_world_fragment -> HiddenWorldFragment;
}
