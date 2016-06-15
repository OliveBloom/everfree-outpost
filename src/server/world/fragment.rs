use types::*;

use world::World;
use world::{Client, Entity, Inventory, Plane, TerrainChunk, Structure};
use world::hooks::{Hooks, NoHooks};
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

    type H: Hooks;
    fn with_hooks<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Self::H) -> R;

    $(

    fn $create_obj<'a>(&'a mut self, $($create_arg: $create_arg_ty,)*)
                   -> OpResult<ObjectRefMut<'a, 'd, $Obj, Self>> {
        #![allow(unused_variables)]  // id_expr may not reference id_name
        let $create_id_name = try!(ops::$module::create(self, $($create_arg,)*));
        Ok(ObjectRefMut::new(self, $create_id_expr))
    }

    fn $destroy_obj(&mut self, id: $Id) -> OpResult<()> {
        ops::$module::destroy(self, id)
    }

    fn $get_obj_mut<'a>(&'a mut self, $lookup_id_name: $Id)
                        -> Option<ObjectRefMut<'a, 'd, $Obj, Self>> {
        match self.world().$objs.get($lookup_id_expr) {
            None => return None,
            Some(_) => {},
        }

        Some(ObjectRefMut::new(self, $lookup_id_name))
    }

    fn $obj_mut<'a>(&'a mut self, id: $Id) -> ObjectRefMut<'a, 'd, $Obj, Self> {
        self.$get_obj_mut(id)
            .expect(concat!("no ", stringify!($Obj), " with given id"))
    }

    )*

    fn create_structure_unchecked<'a>(&'a mut self,
                                      pid: PlaneId,
                                      pos: V3,
                                      tid: TemplateId)
                                      -> OpResult<ObjectRefMut<'a, 'd, Structure, Self>> {
        // Check validity of `pid` and `tid`.
        unwrap!(self.world().get_plane(pid));
        unwrap!(self.world().data.structure_templates.get_template(tid));
        let stable_pid = self.world_mut().planes.pin(pid);

        let sid = ops::structure::create_unchecked(self);
        {
            let s = &mut self.world_mut().structures[sid];

            s.plane = pid;
            s.stable_plane = stable_pid;
            s.pos = pos;
            s.template = tid;
        }
        ops::structure::post_init(self, sid);
        Ok(ObjectRefMut::new(self, sid))
    }
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

    type H = NoHooks;
    fn with_hooks<F, R>(&mut self, f: F) -> R
            where F: FnOnce(&mut NoHooks) -> R {
        f(&mut NoHooks)
    }
}
