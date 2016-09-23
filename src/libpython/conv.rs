use std::collections::HashMap;
use std::hash::Hash;

use api as py;
use api::{PyBox, PyRef, PyResult};


/// Types that can be converted from Python objects.
pub trait Unpack<'a>: Sized {
    fn unpack(obj: PyRef<'a>) -> PyResult<Self>;
}

impl<'a> Unpack<'a> for PyRef<'a> {
    fn unpack(obj: PyRef<'a>) -> PyResult<PyRef<'a>> {
        Ok(obj)
    }
}

impl<'a> Unpack<'a> for PyBox {
    fn unpack(obj: PyRef<'a>) -> PyResult<PyBox> {
        Ok(obj.to_box())
    }
}

impl<'a, A: Unpack<'a>> Unpack<'a> for Option<A> {
    fn unpack(obj: PyRef<'a>) -> PyResult<Option<A>> {
        if obj == py::none() {
            Ok(None)
        } else {
            Ok(Some(try!(Unpack::unpack(obj))))
        }
    }
}

impl<'a> Unpack<'a> for String {
    fn unpack(obj: PyRef<'a>) -> PyResult<String> {
        py::unicode::as_string(obj)
    }
}

impl<'a, T> Unpack<'a> for Vec<T>
        where T: Unpack<'a> {
    fn unpack(obj: PyRef<'a>) -> PyResult<Vec<T>> {
        let len = try!(py::list::size(obj));
        let mut v = Vec::with_capacity(len);
        for i in 0 .. len {
            let item = try!(py::list::get_item(obj, i));
            v.push(try!(Unpack::unpack(item)));
        }
        Ok(v)
    }
}

impl<'a, K, V> Unpack<'a> for HashMap<K, V>
        where K: for<'b> Unpack<'b> + Eq + Hash,
              V: for<'b> Unpack<'b> {
    fn unpack(obj: PyRef<'a>) -> PyResult<HashMap<K, V>> {
        let items = try!(py::dict::items(obj));
        let items_ = items.borrow();

        let len = try!(py::list::size(items_));
        let mut h = HashMap::with_capacity(len);
        for i in 0 .. len {
            let item = try!(py::list::get_item(items_, i));
            let (k, v) = try!(Unpack::unpack(item));
            h.insert(k, v);
        }
        Ok(h)
    }
}

impl<'a> Unpack<'a> for bool {
    fn unpack(obj: PyRef<'a>) -> PyResult<bool> {
        if obj == py::bool::true_() {
            Ok(true)
        } else if obj == py::bool::false_() {
            Ok(false)
        } else {
            pyraise!(type_error, "expected an instance of bool");
        }
    }
}

impl<'a> Unpack<'a> for f64 {
    fn unpack(obj: PyRef<'a>) -> PyResult<f64> {
        py::float::as_f64(obj)
    }
}


/// Types that can be converted to Python objects.
pub trait Pack {
    fn pack(self) -> PyResult<PyBox>;
}

impl<T: Pack> Pack for PyResult<T> {
    fn pack(self) -> PyResult<PyBox> {
        self.and_then(|x| Pack::pack(x))
    }
}

impl Pack for PyBox {
    fn pack(self) -> PyResult<PyBox> {
        Ok(self)
    }
}

impl<'a> Pack for PyRef<'a> {
    fn pack(self) -> PyResult<PyBox> {
        Ok(self.to_box())
    }
}

impl<A: Pack> Pack for Option<A> {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            Some(a) => Pack::pack(a),
            None => Ok(py::none().to_box()),
        }
    }
}

impl<'a> Pack for &'a str {
    fn pack(self) -> PyResult<PyBox> {
        py::unicode::from_str(self)
    }
}

impl Pack for String {
    fn pack(self) -> PyResult<PyBox> {
        py::unicode::from_str(&self)
    }
}

impl<T> Pack for Vec<T>
        where T: Pack {
    fn pack(self) -> PyResult<PyBox> {
        let lst = try!(py::list::new());
        for x in self {
            let x = try!(Pack::pack(x));
            try!(py::list::append(lst.borrow(), x.borrow()));
        }
        Ok(lst)
    }
}

impl<'a, T> Pack for &'a [T]
        where T: Pack+Clone {
    fn pack(self) -> PyResult<PyBox> {
        let lst = try!(py::list::new());
        for x in self {
            let x = try!(Pack::pack(x.clone()));
            try!(py::list::append(lst.borrow(), x.borrow()));
        }
        Ok(lst)
    }
}

impl<K, V> Pack for HashMap<K, V>
        where K: Pack + Eq + Hash,
              V: Pack {
    fn pack(self) -> PyResult<PyBox> {
        let dct = try!(py::dict::new());
        for (k, v) in self {
            let k = try!(Pack::pack(k));
            let v = try!(Pack::pack(v));
            try!(py::dict::set_item(dct.borrow(), k.borrow(), v.borrow()));
        }
        Ok(dct)
    }
}

impl Pack for bool {
    fn pack(self) -> PyResult<PyBox> {
        if self {
            Ok(py::bool::true_().to_box())
        } else {
            Ok(py::bool::false_().to_box())
        }
    }
}

impl Pack for f64 {
    fn pack(self) -> PyResult<PyBox> {
        py::float::from_f64(self)
    }
}


// Macros for generating tuple, integer, and ID impls

macro_rules! tuple_impls {
    ($count:expr, $pack:ident: ($($A:ident $idx:expr),*)) => {
        impl<'a, $($A: Unpack<'a>,)*> Unpack<'a> for ($($A,)*) {
            fn unpack(obj: PyRef<'a>) -> PyResult<($($A,)*)> {
                pyassert!(py::tuple::check(obj), type_error);
                pyassert!(try!(py::tuple::size(obj)) == $count, value_error);
                Ok((
                    $( try!(Unpack::unpack(
                                try!(py::tuple::get_item(obj, $idx)))), )*
                ))
            }
        }

        impl<$($A: Pack,)*> Pack for ($($A,)*) {
            #[allow(non_snake_case)]
            fn pack(self) -> PyResult<PyBox> {
                let ($($A,)*) = self;
                py::tuple::$pack(
                    $(try!(Pack::pack($A)),)*
                )
            }
        }
    };
}

tuple_impls!(0, pack0: ());
tuple_impls!(1, pack1: (A 0));
tuple_impls!(2, pack2: (A 0, B 1));
tuple_impls!(3, pack3: (A 0, B 1, C 2));
tuple_impls!(4, pack4: (A 0, B 1, C 2, D 3));


macro_rules! int_impls {
    ($ty:ident, unsigned) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> PyResult<$ty> {
                let raw = try!(py::int::as_u64(obj));
                pyassert!(raw <= ::std::$ty::MAX as u64, value_error);
                Ok(raw as $ty)
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyResult<PyBox> {
                py::int::from_u64(self as u64)
            }
        }
    };

    ($ty:ident, signed) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> PyResult<$ty> {
                let raw = try!(py::int::as_i64(obj));
                pyassert!(raw >= ::std::$ty::MIN as i64, value_error);
                pyassert!(raw <= ::std::$ty::MAX as i64, value_error);
                Ok(raw as $ty)
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyResult<PyBox> {
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
