use std::marker::PhantomData;
use std::mem;

use types::*;

use data::Data;
use engine::Engine;
use script::ScriptHooks;
use storage::Storage;


macro_rules! EnginePart_decl {
    ($($tv:ident $tv2:ident $tv3:ident ($field:ident, $field_mut:ident, $fty:ty),)*) => {
        pub struct EnginePart<'a, 'd, $($tv: 'a),*> {
            ptr: *mut Engine<'d>,
            _marker0: PhantomData<&'d Data>,
            _marker1: PhantomData<&'d Storage>,
            _marker2: PhantomData<&'d ScriptHooks>,
            $($field: PhantomData<&'a mut $tv>,)*
        }

        pub struct Open<'a, 'd, $($tv: 'a),*> {
            pub data: &'d Data,
            pub storage: &'d Storage,
            pub script_hooks: &'d ScriptHooks,
            $( pub $field: &'a mut $tv, )*
        }

        // Suppress some spurious(?) warnings about SplitOffRHS::RHS being private.
        impl<'a, 'd, $($tv),*> EnginePart<'a, 'd, $($tv),*> {
            unsafe fn from_raw(e: *mut Engine<'d>) -> EnginePart<'a, 'd, $($tv,)*> {
                EnginePart {
                    ptr: e,
                    _marker0: PhantomData,
                    _marker1: PhantomData,
                    _marker2: PhantomData,
                    $($field: PhantomData,)*
                }
            }

            pub fn as_ptr(&self) -> *mut Engine<'d> {
                self.ptr
            }

            pub fn borrow<'b>(&'b mut self) -> EnginePart<'b, 'd, $($tv),*> {
                unsafe { EnginePart::from_raw(self.ptr) }
            }

            pub fn slice<$($tv2),*>(self) -> EnginePart<'a, 'd, $($tv2),*>
                    where EnginePart<'a, 'd, $($tv2),*>: Subpart<Self> {
                unsafe { EnginePart::from_raw(self.ptr) }
            }

            pub fn split<$($tv2, $tv3,)*>(self) ->
                        (EnginePart<'a, 'd, $($tv2),*>,
                         EnginePart<'a, 'd, $($tv3),*>)
                    where (EnginePart<'a, 'd, $($tv2),*>,
                           EnginePart<'a, 'd, $($tv3),*>): Subpart2<Self> {
                unsafe { (EnginePart::from_raw(self.ptr),
                          EnginePart::from_raw(self.ptr)) }
            }

            pub fn split_off<$($tv2,)*>(self) ->
                        (EnginePart<'a, 'd, $($tv2),*>,
                         EnginePart<'a, 'd, $(<$tv as SplitOffRHS<'a, $tv2>>::RHS),*>)
                    where $($tv: SplitOffRHS<'a, $tv2>,)*
                          (EnginePart<'a, 'd, $($tv2),*>,
                           EnginePart<'a, 'd, $(<$tv as SplitOffRHS<'a, $tv2>>::RHS),*>):
                              Subpart2<Self> {
                self.split()
            }

            pub fn open<'b>(&'b mut self) -> Open<'b, 'd, $($tv),*> {
                let data = self.data();
                let storage = self.storage();
                let script_hooks = self.script_hooks();
                unsafe {
                    Open {
                        data: data,
                        storage: storage,
                        script_hooks: script_hooks,
                        $( $field: mem::transmute(&mut (*self.ptr).$field), )*
                    }
                }
            }

            pub fn data(&self) -> &'d Data {
                unsafe { (*self.ptr).data }
            }

            pub fn storage(&self) -> &'d Storage {
                unsafe { (*self.ptr).storage }
            }

            pub fn script_hooks(&self) -> &'d ScriptHooks {
                unsafe { (*self.ptr).script_hooks }
            }

            pub fn now(&self) -> Time {
                unsafe { (*self.ptr).now }
            }

            $(
                pub fn $field<'b>(&'b self) -> &'b $tv {
                    unsafe {
                        mem::transmute(&(*self.ptr).$field)
                    }
                }

                pub fn $field_mut<'b>(&'b mut self) -> &'b mut $tv {
                    unsafe {
                        mem::transmute(&mut (*self.ptr).$field)
                    }
                }
            )*

            pub unsafe fn fiddle<'b: 'a>(self) -> EnginePart<'b, 'd, $($tv),*> {
                EnginePart::from_raw(self.ptr)
            }
        }


        unsafe impl<'a, 'd, $($tv, $tv2,)*> Subpart<EnginePart<'a, 'd, $($tv),*>>
                for EnginePart<'a, 'd, $($tv2),*>
                where $($tv2: Subitem<$tv>),* {}

        unsafe impl<'a, 'd, $($tv, $tv2, $tv3,)*>
                Subpart2<EnginePart<'a, 'd, $($tv),*>>
                for (EnginePart<'a, 'd, $($tv2),*>, EnginePart<'a, 'd, $($tv3),*>)
                where $(($tv2, $tv3): Subitem2<$tv>),* {}

        unsafe impl<'a, 'd, $($tv: ItemFlags,)*> PartFlags for EnginePart<'a, 'd, $($tv),*> {
            fn flags() -> usize {
                let mut x = 0;
                $(
                    x <<= 1;
                    x |= <$tv as ItemFlags>::present() as usize;
                )*
                x
            }
        }

        $( subitem_impls!($fty); )*


        pub trait Part: Sized {
            type P: Sized;
            fn from_part(part: Self::P) -> Self;
            fn to_part(self) -> Self::P;
            unsafe fn from_ptr(ptr: *mut Engine) -> Self;
            fn as_ptr(&self) -> *mut Engine<'static>;
        }
    };
}

macro_rules! subitem_impls {
    ( $t:ty ) => {
        unsafe impl<'d> Subitem<$t> for $t {}
        unsafe impl<'d> Subitem<$t> for () {}

        unsafe impl<'d> Subitem2<$t> for ($t, ()) {}
        unsafe impl<'d> Subitem2<$t> for ((), $t) {}
        unsafe impl<'d> Subitem2<$t> for ((), ()) {}

        unsafe impl<'d> ItemFlags for $t {
            fn present() -> bool { true }
        }

        impl<'a, 'd: 'a> SplitOffRHS<'a, ()> for $t {
            type RHS = $t;
        }

        impl<'a, 'd: 'a> SplitOffRHS<'a, $t> for $t {
            type RHS = ();
        }
    };
}

EnginePart_decl! {
    Wr Wr2 Wr3 (world, world_mut, ::world::World<'d>),
    Ex Ex2 Ex3 (extra, extra_mut, ::logic::extra::Extra),
    Ms Ms2 Ms3 (messages, messages_mut, ::messages::Messages),
    Ti Ti2 Ti3 (timer, timer_mut, ::timer::Timer),
    Ph Ph2 Ph3 (physics, physics_mut, ::physics::Physics<'d>),
    Vi Vi2 Vi3 (vision, vision_mut, ::vision::Vision),
    Au Au2 Au3 (auth, auth_mut, ::auth::Auth),
    Ch Ch2 Ch3 (chunks, chunks_mut, ::chunks::Chunks<'d>),
    Ca Ca2 Ca3 (cache, cache_mut, ::cache::TerrainCache),
    Tg Tg2 Tg3 (terrain_gen, terrain_gen_mut, ::terrain_gen::TerrainGen),
}


pub unsafe trait Subpart<E> {}

pub unsafe trait Subitem<A> {}
unsafe impl Subitem<()> for () {}

pub unsafe trait Subpart2<E> {}

pub unsafe trait Subitem2<A> {}
unsafe impl Subitem2<()> for ((), ()) {}

pub unsafe trait PartFlags {
    fn flags() -> usize;
}

pub unsafe trait ItemFlags {
    fn present() -> bool;
}

unsafe impl ItemFlags for () {
    fn present() -> bool { false }
}


pub trait SplitOffRHS<'a, LHS> {
    type RHS: 'a;
}

impl<'a> SplitOffRHS<'a, ()> for () {
    type RHS = ();
}


macro_rules! engine_part_typedef_pub {
    ($name:ident, $wr:ty, $ex:ty, $ms:ty, $ti:ty, $ph:ty, $vi:ty, $au:ty, $ch:ty, $ca:ty, $tg:ty) => {
        pub struct $name<'a, 'd: 'a>(pub ::engine::split::EnginePart<'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg>);
        engine_part_typedef_impls!($name, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg);
    };
}

macro_rules! engine_part_typedef_priv {
    ($name:ident, $wr:ty, $ex:ty, $ms:ty, $ti:ty, $ph:ty, $vi:ty, $au:ty, $ch:ty, $ca:ty, $tg:ty) => {
        struct $name<'a, 'd: 'a>(pub ::engine::split::EnginePart<'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg>);
        engine_part_typedef_impls!($name, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg);
    };
}

macro_rules! engine_part_typedef_impls {
    ($name:ident, $wr:ty, $ex:ty, $ms:ty, $ti:ty, $ph:ty, $vi:ty, $au:ty, $ch:ty, $ca:ty, $tg:ty) => {
        impl<'a, 'd: 'a> $crate::engine::split::Part for $name<'a, 'd> {
            type P = $crate::engine::split::EnginePart<
                'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg>;

            fn from_part(part: $crate::engine::split::EnginePart<
                             'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg>) -> $name<'a, 'd> {
                $name(part)
            }

            fn to_part(self) -> $crate::engine::split::EnginePart<
                    'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg> {
                self.0
            }

            unsafe fn from_ptr(ptr: *mut $crate::engine::Engine) -> $name<'a, 'd> {
                ::std::mem::transmute(ptr)
            }

            fn as_ptr(&self) -> *mut $crate::engine::Engine<'static> {
                // Shouldn't need a transmute here, but apparently you can't directly cast between
                // *mut T<'a> and *mut T<'b>...
                unsafe { ::std::mem::transmute(self.0.as_ptr()) }
            }
        }

        impl<'a, 'd: 'a> $name<'a, 'd> {
            pub fn borrow<'b>(&'b mut self) -> $name<'b, 'd> {
                $name(self.0.borrow())
            }

            pub unsafe fn fiddle<'b: 'a>(self) -> $name<'b, 'd> {
                $name(self.0.fiddle())
            }
        }

        impl<'a, 'd: 'a> ::std::ops::Deref for $name<'a, 'd> {
            type Target = $crate::engine::split::EnginePart<
                 'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg>;

            fn deref(&self) -> &<Self as ::std::ops::Deref>::Target {
                &self.0
            }
        }

        impl<'a, 'd: 'a> ::std::ops::DerefMut for $name<'a, 'd> {
            fn deref_mut(&mut self) -> &mut <Self as ::std::ops::Deref>::Target {
                &mut self.0
            }
        }

        unsafe impl<'a, 'd: 'a> $crate::engine::split::PartFlags for $name<'a, 'd> {
            fn flags() -> usize {
                <$crate::engine::split::EnginePart<
                        'a, 'd, $wr, $ex, $ms, $ti, $ph, $vi, $au, $ch, $ca, $tg>
                    as $crate::engine::split::PartFlags>::flags()

            }
        }
    };
}

engine_part_typedef!(pub EngineRef);

impl<'a, 'd> EngineRef<'a, 'd> {
    pub fn new(e: &'a mut Engine<'d>) -> Self {
        EngineRef(unsafe { EnginePart::from_raw(e as *mut _) })
    }

    pub fn unwrap(self) -> &'a mut Engine<'d> {
        unsafe { mem::transmute(self.0) }
    }
}
