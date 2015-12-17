use libc;
use python3_sys;


macro_rules! one {
    ( $any:tt ) => { 1 };
}

macro_rules! define_func {
    ( $mac:ident, $fname:ident, $args:tt, , $body:block ) => {
        $mac!($fname, $args, (), $body);
    };
    ( $mac:ident, $fname:ident, $args:tt, $ret_ty:ty, $body:block ) => {
        $mac!($fname, $args, $ret_ty, $body);
    };
}

/// Generate code to initialize a Python class.
macro_rules! define_python_class {
    (
        class $name:ident : $ty:ty {
            type_obj $type_obj:ident;
            initializer $init_name:ident;
            accessor $acc_name:ident;
            method_macro $define_func:ident !;
            $(
                fn $fname:ident $fargs:tt $( -> $fret:ty )* { $( $fbody:tt )* }
            )*
        }

    ) => {
        define_python_class! {
            class $name: $ty {
                type_obj $type_obj;
                initializer $init_name;
                accessor $acc_name;
                method_macro $define_func!;
            members:
            slots:
            methods:
                $(
                    fn $fname $fargs $( -> $fret )* { $($fbody)* }
                )*
            }
        }
    };
    (
        class $name:ident : $ty:ty {
            type_obj $type_obj:ident;
            initializer $init_name:ident;
            accessor $acc_name:ident;
            method_macro $define_func:ident !;

        members:
            $(
                let $mname:ident := $($mpart:ident).+ ;
            )*

        slots:
            $(
                fn($smacro:ident!) $sname:ident $sargs:tt $( -> $sret:ty )* { $( $sbody:tt )* }
            )*

        methods:
            $(
                fn $fname:ident $fargs:tt $( -> $fret:ty )* { $( $fbody:tt )* }
            )*
        }
    ) => {
        static mut $type_obj: *mut ::python3_sys::PyObject = 0 as *mut _;

        #[allow(non_snake_case)]
        pub fn $init_name(module: $crate::python::PyRef) {
            $(
                define_func!($smacro, $sname, $sargs, $( $sret )*, { $( $sbody )* });
            )*
            $(
                define_func!($define_func, $fname, $fargs, $( $fret )*, { $( $fbody )* });
            )*

            #[allow(
                unused_assignments,
                unused_imports,
                unused_mut,
                unused_variables,
                )]
            unsafe fn _impl(module: $crate::python::PyRef) {
                use std::mem;
                use std::ptr;
                use libc::{c_char, c_int, c_uint};

                use python3_sys;
                use python3_sys::{PyType_Spec, PyType_Slot, PyMethodDef};
                use python3_sys::{METH_VARARGS, Py_TPFLAGS_DEFAULT};
                use python3_sys::structmember::{PyMemberDef, READONLY};

                use $crate::script2::{BLANK_TYPE_SPEC, BLANK_TYPE_SLOT};
                use $crate::script2::{BLANK_METHOD_DEF, BLANK_MEMBER_DEF};
                use $crate::script2::class::{decay, get_ptr_type_code};
                use $crate::python as py;

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
                            pfunc: mem::transmute($sname),
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
                            ml_meth: Some(mem::transmute($fname)),
                            ml_flags: METH_VARARGS,
                            ml_doc: ptr::null(),
                        };
                        i += 1;
                    )*
                }

                let type_obj = py::type_::from_spec(&mut TYPE_SPEC);
                py::object::set_attr_str(module, stringify!($name), type_obj.borrow());
                $type_obj = type_obj.unwrap();
            }

            unsafe { _impl(module) };
        }

        pub fn $acc_name() -> $crate::python::PyRef<'static> {
            unsafe {
                $crate::python::PyRef::new($type_obj)
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
