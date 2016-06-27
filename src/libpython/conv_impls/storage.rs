use server_config::Storage;

use api as py;
use api::{PyBox, PyResult};
use rust_ref::{RustRef, RustRefType};

macro_rules! storage_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(Storage, $($all)*); );
}

define_python_class! {
    class StorageRef: RustRef {
        type_obj STORAGE_REF_TYPE;
        initializer init;
        accessor get_type;
        method_macro storage_ref_func!;

        fn script_dir(&this) -> PyResult<PyBox> {
            py::unicode::from_str(this.script_dir().to_str().unwrap())
        }
    }
}

unsafe impl RustRefType for Storage {
    fn get_type_object() -> PyBox {
        get_type().to_box()
    }
}

