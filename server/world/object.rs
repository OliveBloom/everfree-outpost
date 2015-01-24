use std::ops::{Deref, DerefMut};
use std::ptr;

use world::World;


pub trait Object: 'static {
    type Id: Copy;

    fn get<'a>(world: &'a World, id: <Self as Object>::Id) -> Option<&'a Self>;
    fn get_mut<'a>(world: &'a mut World, id: <Self as Object>::Id) -> Option<&'a mut Self>;
}


pub struct ObjectRef<'a, 'd: 'a, O: Object> {
    pub world: &'a mut World<'d>,
    pub id: <O as Object>::Id,
}

pub trait ObjectRefT<'d, O>: Deref<Target=O>+DerefMut {
    fn world<'b>(&'b self) -> &'b World<'d>;
    fn world_mut<'b>(&'b mut self) -> &'b mut World<'d>;
    fn id(&self) -> <O as Object>::Id;
    fn obj<'b>(&'b self) -> &'b O;
    fn obj_mut<'b>(&'b mut self) -> &'b mut O;
}

impl<'a, 'd, O: Object> ObjectRefT<'d, O> for ObjectRef<'a, 'd, O> {
    // NB: We use a shorter lifetime 'b instead of 'a to ensure that the world and the object can't
    // be borrowed at the same time.
    fn world<'b>(&'b self) -> &'b World<'d> {
        &*self.world
    }

    fn world_mut<'b>(&'b mut self) -> &'b mut World<'d> {
        &mut *self.world
    }

    fn id(&self) -> <O as Object>::Id {
        self.id
    }

    fn obj<'b>(&'b self) -> &'b O {
        <O as Object>::get(self.world, self.id)
            .expect("tried to call ObjectRef::obj() after deleting the object")
    }

    fn obj_mut<'b>(&'b mut self) -> &'b mut O {
        <O as Object>::get_mut(self.world, self.id)
            .expect("tried to call ObjectRef::obj_mut() after deleting the object")
    }
}

impl<'a, 'd, O: Object> Deref for ObjectRef<'a, 'd, O> {
    type Target = O;
    fn deref<'b>(&'b self) -> &'b O {
        self.obj()
    }
}

impl<'a, 'd, O: Object> DerefMut for ObjectRef<'a, 'd, O> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut O {
        self.obj_mut()
    }
}
