use python3_sys::*;

use types::*;

use engine::Engine;
use engine::split::{EngineRef, Part, PartFlags};
use python as py;
use python::PyRef;
use script::ScriptEngine;


/// A reference to an `Engine`.  Similar to `RustRef`, but keeps different metadata (engine part
/// flags, instead of mutability).
struct PyEngineRef {
    base: PyObject,
    ptr: *mut Engine<'static>,
    flags: usize,
}

macro_rules! engine_ref_func {
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
                use $crate::engine::split::{Part, PartFlags};
                use $crate::python as py;
                use $crate::python::PyRef;
                use $crate::script2::{Pack, Unpack};

                let slf = PyRef::new(slf);
                let args = PyRef::new(args);

                assert!(py::object::is_instance(slf, get_type()));
                let er = &mut *(slf.as_ptr() as *mut PyEngineRef);
                let ref_flags = er.flags;
                let target_flags = <$aty1 as PartFlags>::flags();
                assert!(ref_flags & target_flags == target_flags);

                er.flags = 0;
                let result = {
                    let target = <$aty1 as Part>::from_ptr(er.ptr);
                    $fname(target, Unpack::unpack(args))
                };
                er.flags = ref_flags;
                Pack::pack(result).unwrap()
            }
        }

    };
}

define_python_class! {
    class EngineRef: PyEngineRef {
        type_obj ENGINE_REF_TYPE;
        initializer init;
        accessor get_type;
        method_macro engine_ref_func!;

        fn now(eng: EmptyPart,) -> Time {
            eng.now()
        }

        fn script_cb_chat_command(eng: EngineRef, cid: ClientId, msg: String) {
            warn_on_err!(ScriptEngine::cb_chat_command(eng.unwrap(), cid, &msg));
        }

        fn messages_clients_len(eng: OnlyMessages,) -> usize {
            eng.messages().clients_len()
        }

        fn messages_send_chat_update(eng: OnlyMessages, cid: ClientId, msg: String) {
            use messages::ClientResponse;
            let resp = ClientResponse::ChatUpdate(msg);
            eng.messages().send_client(cid, resp);
        }
    }
}

pub fn with_engine_ref<E, F, R>(mut e: E, f: F) -> R
        where E: Part + PartFlags, F: FnOnce(PyRef) -> R {
    unsafe {
        let obj = py::type_::instantiate(get_type());
        {
            let er = &mut *(obj.as_ptr() as *mut PyEngineRef);
            er.ptr = e.as_ptr();
            er.flags = <E as PartFlags>::flags();
        }

        let result = f(obj.borrow());

        {
            let er = &mut *(obj.as_ptr() as *mut PyEngineRef);
            er.flags = 0;
        }

        result
    }
}

engine_part_typedef!(OnlyWorld(world));
engine_part_typedef!(OnlyMessages(messages));
engine_part_typedef!(EmptyPart());
