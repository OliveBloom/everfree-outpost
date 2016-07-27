use types::*;

use world::World;
use world::{Client, Entity, Inventory, Plane, TerrainChunk, Structure};
use world::object::ObjectRefMut;
use world::ops::{self, OpResult};

macro_rules! define_Fragment {
    ($(
        object $Obj:ident {
            id $Id:ident;
            map $objs:ident;
            module $module:ident;
            lifecycle ($($create_arg:ident: $create_arg_ty:ty),*)
                $create_obj:ident [$create_id_name:ident -> $create_id_expr:expr],
                $destroy_obj:ident,
            lookups [$lookup_id_name:ident -> $lookup_id_expr:expr]
                $get_obj:ident, $obj:ident,
                $get_obj_mut:ident, $obj_mut:ident,
            $(stable_ids
                $transient_obj_id:ident;)*
        }
    )*) => {

pub trait Fragment<'d>: Sized {
    fn world(&self) -> &World<'d>;
    fn world_mut(&mut self) -> &mut World<'d>;

    $(

    fn $create_obj<'a>(&'a mut self, $($create_arg: $create_arg_ty,)*)
                   -> OpResult<ObjectRefMut<'a, 'd, $Obj>> {
        self.world_mut().$create_obj($($create_arg,)*)
    }

    fn $destroy_obj(&mut self, id: $Id) -> OpResult<()> {
        self.world_mut().$destroy_obj(id)
    }

    fn $get_obj_mut<'a>(&'a mut self, $lookup_id_name: $Id)
                        -> Option<ObjectRefMut<'a, 'd, $Obj>> {
        self.world_mut().$get_obj_mut($lookup_id_name)
    }

    fn $obj_mut<'a>(&'a mut self, id: $Id) -> ObjectRefMut<'a, 'd, $Obj> {
        self.world_mut().$obj_mut(id)
    }

    )*
}

    }
}

process_objects!(define_Fragment!);


pub struct DummyFragment<'a, 'd: 'a> {
    w: &'a mut World<'d>,
}

impl<'a, 'd> DummyFragment<'a, 'd> {
    pub fn new(w: &'a mut World<'d>) -> DummyFragment<'a, 'd> {
        DummyFragment {
            w: w,
        }
    }
}

impl<'a, 'd> Fragment<'d> for DummyFragment<'a, 'd> {
    fn world(&self) -> &World<'d> {
        self.w
    }

    fn world_mut(&mut self) -> &mut World<'d> {
        self.w
    }
}
