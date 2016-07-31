use libc;
use python3_sys;

use python3_sys::{PyType_Spec, PyType_Slot, PyMethodDef};
use python3_sys::structmember::PyMemberDef;


pub const BLANK_TYPE_SPEC: PyType_Spec = PyType_Spec {
    name: 0 as *const _,
    basicsize: 0,
    itemsize: 0,
    flags: 0,
    slots: 0 as *mut _,
};

pub const BLANK_TYPE_SLOT: PyType_Slot = PyType_Slot {
    slot: 0,
    pfunc: 0 as *mut _,
};

pub const BLANK_MEMBER_DEF: PyMemberDef = PyMemberDef {
    name: 0 as *mut _,
    type_code: 0,
    offset: 0,
    flags: 0,
    doc: 0 as *mut _,
};

pub const BLANK_METHOD_DEF: PyMethodDef = PyMethodDef {
    ml_name: 0 as *const _,
    ml_meth: None,
    ml_flags: 0,
    ml_doc: 0 as *const _,
};

#[macro_export]
macro_rules! one {
    ( $any:tt ) => { 1 };
}

/// Backend implementation for `define_python_class!` from `libsyntax_exts`.
#[macro_export]
macro_rules! define_python_class_impl {
    (
        $name:ident, $ty:ty,
        $type_obj:ident, $init_name:ident, $acc_name:ident,
        // Methods
        { $( $fmacro:ident, $fname:ident, $fargs:tt, $fret:ty, $fbody:expr; )* }
        // Slots
        { $( $smacro:ident, $sname:ident, $sargs:tt, $sret:ty, $sbody:expr; )* }
        // Members
        { $( $mname:ident, $($mpart:ident).+; )* }
    ) => {
        static mut $type_obj: *mut ::python3_sys::PyObject = 0 as *mut _;

        #[allow(non_snake_case)]
        pub fn $init_name(module: $crate::ptr::PyRef) {
            $( $smacro!($sname, $sargs, $sret, $sbody); )*
            $( $fmacro!($fname, $fargs, $fret, $fbody); )*

            #[allow(
                unused_assignments,
                unused_imports,
                unused_mut,
                unused_variables,
                )]
            unsafe fn _impl(module: $crate::ptr::PyRef) {
                use std::mem;
                use std::ptr;
                use libc::{c_char, c_int, c_uint};

                use python3_sys;
                use python3_sys::{PyType_Spec, PyType_Slot, PyMethodDef};
                use python3_sys::{METH_VARARGS, Py_TPFLAGS_DEFAULT};
                use python3_sys::structmember::{PyMemberDef, READONLY};

                use $crate::class::{BLANK_TYPE_SPEC, BLANK_TYPE_SLOT};
                use $crate::class::{BLANK_METHOD_DEF, BLANK_MEMBER_DEF};
                use $crate::class::{decay, get_ptr_type_code};
                use $crate::api as py;

                assert!(py::is_initialized());

                const NUM_MEMBERS: usize = 0 $( + one!($mname) )*;
                // Extra slots for Py_tp_methods + Py_tp_members
                const NUM_SLOTS: usize = 2 $( + one!($sname) )*;
                const NUM_METHODS: usize = 0 $( + one!($fname) )*;

                static mut TYPE_SPEC: PyType_Spec = BLANK_TYPE_SPEC;
                static mut TYPE_SLOTS: [PyType_Slot; 1 + NUM_SLOTS] =
                        [BLANK_TYPE_SLOT; 1 + NUM_SLOTS];
                static mut MEMBERS: [PyMemberDef; 1 + NUM_MEMBERS] =
                        [BLANK_MEMBER_DEF; 1 + NUM_MEMBERS];
                static mut METHODS: [PyMethodDef; 1 + NUM_METHODS] =
                        [BLANK_METHOD_DEF; 1 + NUM_METHODS];

                TYPE_SPEC = PyType_Spec {
                    name: concat!("_outpost_server.", stringify!($name), "\0")
                            .as_ptr() as *const c_char,
                    basicsize: mem::size_of::<$ty>() as c_int,
                    itemsize: 0,
                    flags: Py_TPFLAGS_DEFAULT as c_uint,
                    slots: TYPE_SLOTS.as_mut_ptr(),
                };

                {
                    // Init MEMBERS
                    let mut i = 0;
                    $({
                        let off_ptr = decay(&mut (*(0 as *mut $ty)).$($mpart).+);
                        MEMBERS[i] = PyMemberDef {
                            name: concat!(stringify!($mname), "\0")
                                    .as_ptr() as *mut c_char,
                            type_code: get_ptr_type_code(off_ptr),
                            offset: off_ptr as Py_ssize_t,
                            flags: 0,
                            doc: ptr::null_mut(),
                        };
                        i += 1;
                    })*
                }

                {
                    // Init TYPE_SLOTS
                    let mut i = 0;

                    TYPE_SLOTS[0] = PyType_Slot {
                        slot: python3_sys::Py_tp_methods,
                        pfunc: METHODS.as_mut_ptr() as *mut _,
                    };
                    i += 1;

                    TYPE_SLOTS[1] = PyType_Slot {
                        slot: python3_sys::Py_tp_members,
                        pfunc: MEMBERS.as_mut_ptr() as *mut _,
                    };
                    i += 1;

                    $(
                        TYPE_SLOTS[i] = PyType_Slot {
                            slot: python3_sys::$sname,
                            // This is a hack, but it works.  Slot macros, when called with no
                            // arguments, should produce the type of the function they define, so
                            // that we can perform the necessary cast here.
                            pfunc: mem::transmute($sname as $smacro!()),
                        };
                        i += 1;
                    )*
                }

                {
                    let mut i = 0;
                    $(
                        METHODS[i] = PyMethodDef {
                            ml_name: concat!(stringify!($fname), "\0")
                                    .as_ptr() as *const c_char,
                            ml_meth: Some(mem::transmute(
                                    $fname as unsafe extern "C" fn(_, _) -> _)),
                            ml_flags: METH_VARARGS,
                            ml_doc: ptr::null(),
                        };
                        i += 1;
                    )*
                }

                let type_obj = py::type_::from_spec(&mut TYPE_SPEC).unwrap();
                py::object::set_attr_str(module, stringify!($name), type_obj.borrow()).unwrap();
                $type_obj = type_obj.unwrap();
            }

            unsafe { _impl(module) };
        }

        pub fn $acc_name() -> $crate::ptr::PyRef<'static> {
            unsafe {
                $crate::ptr::PyRef::new_non_null($type_obj)
            }
        }
    };
}


pub unsafe trait MemberType {
    fn get_type_code() -> libc::c_int;
}

macro_rules! impl_member_type {
    ($($ty:ident => $val:ident,)*) => {
        $(
            unsafe impl MemberType for libc::$ty {
                fn get_type_code() -> libc::c_int {
                    python3_sys::structmember::$val
                }
            }
        )*
    }
}

impl_member_type! {
    c_char => T_BYTE,
    c_uchar => T_UBYTE,
    c_short => T_SHORT,
    c_ushort => T_USHORT,
    c_int => T_INT,
    c_uint => T_UINT,
    // usually c_long == c_int
    //c_long => T_LONG,
    //c_ulong => T_ULONG,
    c_longlong => T_LONGLONG,
    c_ulonglong => T_ULONGLONG,
    c_float => T_FLOAT,
    c_double => T_DOUBLE,
}

pub fn decay<T>(ptr: &mut T) -> *mut T {
    ptr as *mut T
}

pub fn get_ptr_type_code<T: MemberType>(_: *mut T) -> libc::c_int {
    <T as MemberType>::get_type_code()
}


#[macro_export]
macro_rules! method_imp0 {
    ( $imp:ident,
      ( $( $aname:ident : $aty:ty, )*),
      $ret_ty:ty,
      $body:expr ) => {
        fn $imp(($($aname,)*): ($($aty,)*)) -> $ret_ty {
            $body
        }
    };
    ( $imp:ident, ( $( $aname:ident : $aty:ty ),* ), $ret_ty:ty, $body:expr ) => {
        method_imp0!($imp, ($($aname: $aty,)*), $ret_ty, $body);
    };
}

#[macro_export]
macro_rules! method_imp1 {
    ( $imp:ident,
      ( $aname1:ident : $aty1:ty,
        $( $aname:ident : $aty:ty, )*),
      $ret_ty:ty,
      $body:expr ) => {
        fn $imp($aname1: $aty1,
                ($($aname,)*): ($($aty,)*)) -> $ret_ty {
            $body
        }
    };
    ( $imp:ident, ( $( $aname:ident : $aty:ty ),* ), $ret_ty:ty, $body:expr ) => {
        method_imp1!($imp, ($($aname: $aty,)*), $ret_ty, $body);
    };
}

#[macro_export]
macro_rules! method_imp2 {
    ( $imp:ident,
      ( $aname1:ident : $aty1:ty,
        $aname2:ident : $aty2:ty,
        $( $aname:ident : $aty:ty, )*),
      $ret_ty:ty,
      $body:expr ) => {
        fn $imp($aname1: $aty1,
                $aname2: $aty2,
                ($($aname,)*): ($($aty,)*)) -> $ret_ty {
            $body
        }
    };
    ( $imp:ident, ( $( $aname:ident : $aty:ty ),* ), $ret_ty:ty, $body:expr ) => {
        method_imp2!($imp, ($($aname: $aty,)*), $ret_ty, $body);
    };
}

#[macro_export]
macro_rules! call_wrapper {
    ( $wrap:ident, $slf:ident, $args:ident ) => {
        {
            use $crate::exc::return_result;
            use $crate::ptr::PyRef;
            let slf = PyRef::new_non_null($slf);
            let args = PyRef::new_non_null($args);
            return_result($wrap(slf, args))
        }
    };
}

#[macro_export]
macro_rules! default_wrapper {
    ( $wrap:ident, $imp:ident ) => {
        fn $wrap(slf: $crate::ptr::PyRef,
                 args: $crate::ptr::PyRef)
                 -> $crate::exc::PyResult<$crate::ptr::PyBox> {
            use $crate::conv::{Pack, Unpack};
            let result = $imp(try!(Unpack::unpack(slf)),
                              try!(Unpack::unpack(args)));
            Pack::pack(result)
        }
    };
}

#[macro_export]
macro_rules! raw_func {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, $args, $ret_ty, $body);
            default_wrapper!(wrap, imp);
            call_wrapper!(wrap, slf, args)
        }
    };
}

#[macro_export]
macro_rules! default_method {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, $args, $ret_ty, $body);
            default_wrapper!(wrap, imp);
            call_wrapper!(wrap, slf, args)
        }
    };
}
