use python3_sys::*;

use types::*;

use python as py;
use python::{PyBox, PyRef};
use script2::{Pack, Unpack};

pub struct RustVal<T: RustValType> {
    base: PyObject,
    val: T,
}

pub unsafe trait RustValType: Copy {
    fn get_type_object() -> PyBox;
}


pub fn pack_rust_val<T: RustValType>(val: T) -> PyBox {
    unsafe {
        let obj = py::type_::instantiate(T::get_type_object().borrow());
        let r = &mut *(obj.as_ptr() as *mut RustVal<T>);
        r.val = val;
        obj
    }
}

pub fn is_rust_val<T: RustValType>(obj: PyRef) -> bool {
    py::object::is_instance(obj, T::get_type_object().borrow())
}

pub fn unpack_rust_val<T: RustValType>(obj: PyRef) -> T {
    unpack_rust_val_opt(obj).unwrap()
}

pub fn unpack_rust_val_opt<T: RustValType>(obj: PyRef) -> Option<T> {
    if !is_rust_val::<T>(obj) {
        return None;
    }

    let result = unsafe {
        let r = &mut *(obj.as_ptr() as *mut RustVal<T>);
        r.val
    };
    Some(result)
}

macro_rules! rust_val_func {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, $( $aname:ident : $aty:ty ),* ),
      $ret_ty:ty,
      $body:block ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            fn $fname($aname1: $aty1, ($($aname,)*): ($($aty,)*)) -> $ret_ty {
                $body
            }

            {
                use $crate::python::PyRef;
                use $crate::script2::{Pack, Unpack};

                let slf = PyRef::new(slf);
                let args = PyRef::new(args);

                let result = $fname(Unpack::unpack(slf), Unpack::unpack(args));
                Pack::pack(result).unwrap()
            }
        }
    };
}

macro_rules! rust_val_repr_slot {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, ),
      $ret_ty:ty,
      $body:block ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            fn $fname($aname1: $aty1) -> $ret_ty {
                $body
            }

            {
                use $crate::python::PyRef;
                use $crate::script2::{Pack, Unpack};

                let slf = PyRef::new(slf);

                let result = $fname(Unpack::unpack(slf));
                Pack::pack(result).unwrap()
            }
        }
    };
}


define_python_class! {
    class V3: RustVal<V3> {
        type_obj V3_TYPE;
        initializer init_v3;
        accessor get_v3_type;
        method_macro rust_val_func!;

    members:
        let x := val.x;
        let y := val.y;
        let z := val.z;

    slots:
        fn(rust_val_repr_slot!) Py_tp_repr(this: V3,) -> String {
            format!("V3{:?}", this)
        }

    methods:
    }
}

unsafe impl RustValType for V3 {
    fn get_type_object() -> PyBox {
        get_v3_type().to_box()
    }
}

impl Pack for V3 {
    fn pack(self) -> PyBox {
        pack_rust_val(self)
    }
}

impl<'a> Unpack<'a> for V3 {
    fn unpack(obj: PyRef<'a>) -> V3 {
        unpack_rust_val(obj)
    }
}


pub fn init(module: PyRef) {
    init_v3(module);
}
