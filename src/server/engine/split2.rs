use std::marker::PhantomData;
use std::mem;

use types::*;

use data::Data;
use engine::Engine;
use storage::Storage;
use script::ScriptHooks;


pub struct Static<'d> {
    pub data: &'d Data,
    pub storage: &'d Storage,
    pub script_hooks: &'d ScriptHooks,
    pub now: Time,
}

// The general scheme of the generated structs looks like this:
//
//  pub Engine<'d> {
//      _data: &'d Data,
//      _storage: &'d Storage,
//      _script_hooks: &'d ScriptHooks,
//      _now: Time,
//
//      pub world: World<'d>,
//
//      pub extra: Extra,
//      pub messages: Messages,
//      _timer: Timer,
//      pub physics: Physics<'d>,
//      _vision: Vision,
//      _chunks: Chunks<'d>,
//      pub cache: TerrainCache,
//      pub terrain_gen: TerrainGen,
//      _chat: Chat,
//
//      pub input: Input,
//  }
//
// "Available" fields are `pub`, the rest are non-pub and prefixed with an underscore (so that the
// error appears during an earlier compilation pass).  The static fields (data .. now) are all
// private, so that they can't be written by either half after a split.

macro_rules! engine_part2_impl {
    ($name:ident) => {
        impl<'d> $name<'d> {
            pub fn data(&self) -> &'d ::data::Data {
                self._data
            }

            pub fn storage(&self) -> &'d ::storage::Storage {
                self._storage
            }

            pub fn script_hooks(&self) -> &'d ::script::ScriptHooks {
                self._script_hooks
            }

            pub fn now(&self) -> ::types::Time {
                self._now
            }

            pub fn open_static(&self) -> ::engine::split2::Static<'d> {
                ::engine::split2::Static {
                    data: self._data,
                    storage: self._storage,
                    script_hooks: self._script_hooks,
                    now: self._now,
                }
            }
        }
    };
}



pub struct Y<T>(PhantomData<T>);
pub struct N<T>(PhantomData<T>);
pub struct E;


pub trait BitList {
    fn code() -> u64;
    fn len() -> usize;
}

impl<T: BitList> BitList for Y<T> {
    fn code() -> u64 {
        (<T as BitList>::code() << 1) | 1
    }

    fn len() -> usize {
        <T as BitList>::len() + 1
    }
}

impl<T: BitList> BitList for N<T> {
    fn code() -> u64 {
        (<T as BitList>::code() << 1) | 0
    }

    fn len() -> usize {
        <T as BitList>::len() + 1
    }
}

impl BitList for E {
    fn code() -> u64 {
        0
    }

    fn len() -> usize {
        0
    }
}


pub unsafe trait RefineTo<Target: BitList>: BitList {
    type Remnant: BitList;
}
unsafe impl<A, B: BitList> RefineTo<Y<B>> for Y<A> where A: RefineTo<B> {
    type Remnant = N<<A as RefineTo<B>>::Remnant>;
}
unsafe impl<A, B: BitList> RefineTo<N<B>> for Y<A> where A: RefineTo<B> {
    type Remnant = Y<<A as RefineTo<B>>::Remnant>;
}
unsafe impl<A, B: BitList> RefineTo<N<B>> for N<A> where A: RefineTo<B> {
    type Remnant = N<<A as RefineTo<B>>::Remnant>;
}
unsafe impl RefineTo<E> for E {
    type Remnant = E;
}


pub unsafe trait Coded: Sized {
    type Code: BitList;

    fn refine<'a, Target: Coded>(&'a mut self) -> &'a mut Target
            where Self::Code: RefineTo<Target::Code> {
        unsafe { mem::transmute(self) }
    }

    fn split<'a, Target: Coded, Rest: Coded>(&'a mut self) -> (&'a mut Target, &'a mut Rest)
            where Self::Code: RefineTo<Target::Code>,
                  <Self::Code as RefineTo<Target::Code>>::Remnant: RefineTo<Rest::Code> {
        let ptr = self as *mut Self;
        unsafe { (mem::transmute(ptr), mem::transmute(ptr)) }
    }

    fn refine_dynamic<'a, Target: Coded>(&'a mut self) -> Option<&'a mut Target> {
        if <Self::Code as BitList>::len() != <Target::Code as BitList>::len() {
            return None;
        }

        let self_code = <Self::Code as BitList>::code();
        let target_code = <Target::Code as BitList>::code();
        if self_code & target_code == target_code {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }
}

unsafe impl<'d> Coded for Engine<'d> {
    type Code = Y<Y<Y<Y<Y<Y<Y<Y<Y<Y<Y<Y<E>>>>>>>>>>>>;
}
