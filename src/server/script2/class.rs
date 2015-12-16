macro_rules! one {
    ( $any:tt ) => { 1 };
}

macro_rules! define_func {
    ( $mac:ident, $fname:ident, $args:tt, , $body:block ) => {
        $mac!($fname, $args, (), $body)
    };
    ( $mac:ident, $fname:ident, $args:tt, $ret_ty:ty, $body:block ) => {
        $mac!($fname, $args, $ret_ty, $body)
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
                fn $fname:ident $args:tt $( -> $ret_ty:ty )* { $( $body:tt )* }
            )*
        }
    ) => {
        static mut $type_obj: *mut ::python3_sys::PyObject = 0 as *mut _;

        #[allow(unused_assignments)]
        pub fn $init_name(module: $crate::python::PyRef) {
            $(
                define_func!($define_func, $fname, $args, $( $ret_ty )*, { $( $body )* });
            )*
            unsafe {
                use std::mem;
                use std::ptr;
                use libc::{c_char, c_int, c_uint};

                use python3_sys::*;

                use $crate::script2::{BLANK_TYPE_SPEC, BLANK_TYPE_SLOT, BLANK_METHOD_DEF};
                use $crate::python as py;

                assert!(py::is_initialized());

                const NUM_METHODS: usize = 0 $( + one!($fname) )*;
                static mut TYPE_SPEC: PyType_Spec = BLANK_TYPE_SPEC;
                static mut TYPE_SLOTS: [PyType_Slot; 2] = [BLANK_TYPE_SLOT; 2];
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

                TYPE_SLOTS[0] = PyType_Slot {
                    slot: Py_tp_methods,
                    pfunc: METHODS.as_mut_ptr() as *mut _,
                };

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

                let type_obj = py::type_::from_spec(&mut TYPE_SPEC);
                py::object::set_attr_str(module, stringify!($name), type_obj.borrow());
                $type_obj = type_obj.unwrap();
            }
        }

        pub fn $acc_name() -> $crate::python::PyRef<'static> {
            unsafe {
                $crate::python::PyRef::new($type_obj)
            }
        }
    };
}
