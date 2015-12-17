use std::collections::HashMap;
use std::hash::Hash;

use types::*;

use python as py;
use python::{PyBox, PyRef};


/// Types that can be converted from Python objects.
pub trait Unpack<'a> {
    fn unpack(obj: PyRef<'a>) -> Self;
}

impl<'a> Unpack<'a> for PyRef<'a> {
    fn unpack(obj: PyRef<'a>) -> PyRef<'a> {
        obj
    }
}

impl<'a> Unpack<'a> for PyBox {
    fn unpack(obj: PyRef<'a>) -> PyBox {
        obj.to_box()
    }
}

impl<'a> Unpack<'a> for String {
    fn unpack(obj: PyRef<'a>) -> String {
        py::unicode::as_string(obj)
    }
}

impl<'a, T> Unpack<'a> for Stable<T> {
    fn unpack(obj: PyRef<'a>) -> Stable<T> {
        Stable::new(Unpack::unpack(obj))
    }
}


/// Types that can be converted to Python objects.
pub trait Pack {
    fn pack(self) -> PyBox;
}

impl Pack for PyBox {
    fn pack(self) -> PyBox {
        self
    }
}

impl<'a> Pack for PyRef<'a> {
    fn pack(self) -> PyBox {
        self.to_box()
    }
}

impl<A: Pack> Pack for Option<A> {
    fn pack(self) -> PyBox {
        match self {
            Some(a) => Pack::pack(a),
            None => py::none().to_box(),
        }
    }
}

impl<'a> Pack for &'a str {
    fn pack(self) -> PyBox {
        py::unicode::from_str(self)
    }
}

impl Pack for String {
    fn pack(self) -> PyBox {
        py::unicode::from_str(&self)
    }
}

impl<'a, K, V> Pack for &'a HashMap<K, V>
        where K: Clone + Pack + Eq + Hash,
              V: Clone + Pack {
    fn pack(self) -> PyBox {
        let dct = py::dict::new();
        for (k, v) in self {
            let k = Pack::pack(k.clone());
            let v = Pack::pack(v.clone());
            py::dict::set_item(dct.borrow(), k.borrow(), v.borrow());
        }
        dct
    }
}

impl<T> Pack for Stable<T> {
    fn pack(self) -> PyBox {
        Pack::pack(self.unwrap())
    }
}


// Macros for generating tuple, integer, and ID impls

macro_rules! tuple_impls {
    ($count:expr, $pack:ident: ($($A:ident $idx:expr),*)) => {
        impl<'a, $($A: Unpack<'a>,)*> Unpack<'a> for ($($A,)*) {
            fn unpack(obj: PyRef<'a>) -> ($($A,)*) {
                assert!(py::tuple::check(obj));
                assert!(py::tuple::size(obj) == $count);
                (
                    $( Unpack::unpack(py::tuple::get_item(obj, $idx)), )*
                )
            }
        }

        impl<$($A: Pack,)*> Pack for ($($A,)*) {
            #[allow(non_snake_case)]
            fn pack(self) -> PyBox {
                let ($($A,)*) = self;
                py::tuple::$pack(
                    $(Pack::pack($A),)*
                )
            }
        }
    };
}

tuple_impls!(0, pack0: ());
tuple_impls!(1, pack1: (A 0));
tuple_impls!(2, pack2: (A 0, B 1));
tuple_impls!(3, pack3: (A 0, B 1, C 2));


macro_rules! int_impls {
    ($ty:ident, unsigned) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> $ty {
                let raw = py::int::as_u64(obj);
                assert!(raw <= ::std::$ty::MAX as u64);
                raw as $ty
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyBox {
                py::int::from_u64(self as u64)
            }
        }
    };

    ($ty:ident, signed) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> $ty {
                let raw = py::int::as_i64(obj);
                assert!(raw >= ::std::$ty::MIN as i64);
                assert!(raw <= ::std::$ty::MAX as i64);
                raw as $ty
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyBox {
                py::int::from_i64(self as i64)
            }
        }
    };
}

int_impls!(u8, unsigned);
int_impls!(u16, unsigned);
int_impls!(u32, unsigned);
int_impls!(u64, unsigned);
int_impls!(usize, unsigned);

int_impls!(i8, signed);
int_impls!(i16, signed);
int_impls!(i32, signed);
int_impls!(i64, signed);
int_impls!(isize, signed);


macro_rules! id_impls {
    ($ty:ident) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> $ty {
                $ty(Unpack::unpack(obj))
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyBox {
                Pack::pack(self.unwrap())
            }
        }
    };
}

id_impls!(WireId);
id_impls!(ClientId);
id_impls!(EntityId);
id_impls!(InventoryId);
id_impls!(PlaneId);
id_impls!(TerrainChunkId);
id_impls!(StructureId);
