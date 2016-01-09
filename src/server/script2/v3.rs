use std::error::Error;
use python3_sys::*;

use libphysics::TILE_SIZE;

use types::*;

use python as py;
use python::{PyBox, PyRef, PyResult};
use script2::{Pack, Unpack};

pub struct RustVal<T: RustValType> {
    base: PyObject,
    val: T,
}

pub unsafe trait RustValType: Copy {
    fn get_type_object() -> PyBox;
}


pub fn pack_rust_val<T: RustValType>(val: T) -> PyResult<PyBox> {
    unsafe {
        let obj = try!(py::type_::instantiate(T::get_type_object().borrow()));
        let r = &mut *(obj.as_ptr() as *mut RustVal<T>);
        r.val = val;
        Ok(obj)
    }
}

pub fn is_rust_val<T: RustValType>(obj: PyRef) -> bool {
    py::object::is_instance(obj, T::get_type_object().borrow())
}

pub fn unpack_rust_val<T: RustValType>(obj: PyRef) -> PyResult<T> {
    pyassert!(is_rust_val::<T>(obj), type_error);
    let result = unsafe {
        let r = &mut *(obj.as_ptr() as *mut RustVal<T>);
        r.val
    };
    Ok(result)
}

macro_rules! rust_val_repr_slot {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, $args, $ret_ty, $body);

            fn wrap(slf: $crate::python::PyRef)
                    -> $crate::python::PyResult<$crate::python::PyBox> {
                use $crate::script2::{Pack, Unpack};
                let result = imp(try!(Unpack::unpack(slf)), ());
                Pack::pack(result)
            }

            {
                use $crate::python as py;
                use $crate::python::PyRef;
                let slf = PyRef::new_non_null(slf);
                py::return_result(wrap(slf))
            }
        }
    };
}

/// NB: return type must be identical to the class representation type ($ty)
macro_rules! rust_val_init_slot {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject,
                                    _kwds: *mut ::python3_sys::PyObject)
                                    -> ::libc::c_int {
            method_imp0!(imp, $args, $ret_ty, $body);

            {
                use $crate::python::PyRef;
                use $crate::script2::Unpack;

                let args = PyRef::new_non_null(args);
                let rust_args = match Unpack::unpack(args) {
                    Ok(x) => x,
                    Err(e) => {
                        warn!("error in init slot: {}", e.description());
                        return -1;
                    }
                };
                let result = imp(rust_args);
                {
                    let r = &mut *(slf as *mut RustVal<$ret_ty>);
                    r.val = result;
                }
                0
            }
        }
    };
}

/// Ignores all arguments and body, and just calls `PyType_GenericNew`.
macro_rules! rust_val_new_slot {
    ( $fname:ident,
      $args:tt,
      $ret_ty:ty,
      $body:expr ) => {
        unsafe extern "C" fn $fname(subtype: *mut ::python3_sys::PyTypeObject,
                                    args: *mut ::python3_sys::PyObject,
                                    kwds: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            PyType_GenericNew(subtype, args, kwds)
        }
    };
}

fn unpack_v3_or_scalar(obj: PyRef) -> PyResult<Option<V3>> {
    let result: V3 =
        if py::int::check(obj) {
            scalar(try!(Unpack::unpack(obj)))
        } else if py::object::is_instance(obj, get_v3_type()) {
            try!(Unpack::unpack(obj))
        } else {
            return Ok(None);
        };
    Ok(Some(result))
}

macro_rules! v3_binop_slot {
    ( $fname:ident,
      $args:tt,
      $ret_ty:ty,
      $body:expr ) => {
        unsafe extern "C" fn $fname(obj1: *mut ::python3_sys::PyObject,
                                    obj2: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp2!(imp, $args, $ret_ty, $body);

            fn wrap(obj1: $crate::python::PyRef,
                    obj2: $crate::python::PyRef)
                    -> $crate::python::PyResult<$crate::python::PyBox> {
                use $crate::script2::Pack;

                let a = unwrap_or!(try!(unpack_v3_or_scalar(obj1)),
                                   return Ok(py::not_implemented().to_box()));
                let b = unwrap_or!(try!(unpack_v3_or_scalar(obj2)),
                                   return Ok(py::not_implemented().to_box()));

                let result = imp(a, b, ());
                Pack::pack(result)
            }

            {
                use $crate::python as py;
                use $crate::python::PyRef;
                let obj1 = PyRef::new_non_null(obj1);
                let obj2 = PyRef::new_non_null(obj2);
                py::return_result(wrap(obj1, obj2))
            }
        }
    };
}


define_python_class! {
    class V3: RustVal<V3> {
        type_obj V3_TYPE;
        initializer init_v3;
        accessor get_v3_type;
        method_macro default_method!;

    members:
        let x := val.x;
        let y := val.y;
        let z := val.z;

    slots:
        fn(rust_val_repr_slot!) Py_tp_repr(this: V3,) -> String {
            format!("V3{:?}", this)
        }

        fn(rust_val_new_slot!) Py_tp_new() -> () { }

        fn(rust_val_init_slot!) Py_tp_init(x: i32, y: i32, z: i32) -> V3 {
            V3::new(x, y, z)
        }

        fn(v3_binop_slot!) Py_nb_add(a: V3, b: V3) -> V3 { a + b }
        fn(v3_binop_slot!) Py_nb_subtract(a: V3, b: V3) -> V3 { a - b }
        fn(v3_binop_slot!) Py_nb_multiply(a: V3, b: V3) -> V3 { a * b }
        fn(v3_binop_slot!) Py_nb_floor_divide(a: V3, b: V3) -> V3 { a / b }
        fn(v3_binop_slot!) Py_nb_remainder(a: V3, b: V3) -> V3 { a % b }

    methods:
        fn px_to_tile(this: V3,) -> V3 {
            this.div_floor(scalar(TILE_SIZE))
        }

        fn div_floor(this: V3, other: V3) -> V3 {
            this.div_floor(other)
        }
    }
}

unsafe impl RustValType for V3 {
    fn get_type_object() -> PyBox {
        get_v3_type().to_box()
    }
}

impl Pack for V3 {
    fn pack(self) -> PyResult<PyBox> {
        pack_rust_val(self)
    }
}

impl<'a> Unpack<'a> for V3 {
    fn unpack(obj: PyRef<'a>) -> PyResult<V3> {
        unpack_rust_val(obj)
    }
}


pub fn init(module: PyRef) {
    init_v3(module);
}
