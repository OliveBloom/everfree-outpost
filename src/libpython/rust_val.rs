use python3_sys::*;

use api as py;
use api::{PyBox, PyRef, PyResult};


pub struct RustVal<T: RustValType> {
    #[allow(dead_code)]
    base: PyObject,
    pub val: T,
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

#[macro_export]
macro_rules! rust_val_repr_slot {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, $args, $ret_ty, $body);

            fn wrap(slf: $crate::ptr::PyRef)
                    -> $crate::exc::PyResult<$crate::ptr::PyBox> {
                use $crate::conv::{Pack, Unpack};
                let result = imp(try!(Unpack::unpack(slf)), ());
                Pack::pack(result)
            }

            {
                use $crate::exc::return_result;
                use $crate::ptr::PyRef;
                let slf = PyRef::new_non_null(slf);
                return_result(wrap(slf))
            }
        }
    };
}

/// NB: return type must be identical to the class representation type ($ty)
#[macro_export]
macro_rules! rust_val_init_slot {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject,
                                    _kwds: *mut ::python3_sys::PyObject)
                                    -> ::libc::c_int {
            method_imp0!(imp, $args, $ret_ty, $body);

            {
                use $crate::conv::Unpack;
                use $crate::ptr::PyRef;
                use $crate::rust_val::RustVal;

                let args = PyRef::new_non_null(args);
                let rust_args = match Unpack::unpack(args) {
                    Ok(x) => x,
                    Err(e) => {
                        use std::error::Error;
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
#[macro_export]
macro_rules! rust_val_new_slot {
    ( $fname:ident,
      $args:tt,
      $ret_ty:ty,
      $body:expr ) => {
        unsafe extern "C" fn $fname(subtype: *mut ::python3_sys::PyTypeObject,
                                    args: *mut ::python3_sys::PyObject,
                                    kwds: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            ::python3_sys::PyType_GenericNew(subtype, args, kwds)
        }
    };
}
