use types::*;

use engine::Engine;
use engine::split;
use python as py;
use python::{PyBox, PyRef};

use super::{Pack, Unpack};
use super::rust_ref::{RustRef, GetTypeObject};
use super::engine::with_engine_ref;


macro_rules! hooks_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(ScriptHooks, $($all)*) );
}

macro_rules! define_script_hooks {
    ($($name:ident,)*) => {
        pub struct ScriptHooks {
            $($name: Option<PyBox>,)*
        }

        impl ScriptHooks {
            pub fn new() -> ScriptHooks {
                ScriptHooks {
                    $($name: None,)*
                }
            }
        }

        define_python_class! {
            class HooksRef: RustRef {
                type_obj HOOKS_REF_TYPE;
                initializer init;
                accessor get_type;
                method_macro hooks_ref_func!;

                $(
                    fn $name(&mut this, f: PyBox) {
                        this.$name = Some(f);
                    }
                )*
            }
        }
    };
}

define_script_hooks!(
    server_startup, server_shutdown,
    eval,

    client_chat_command,
);

impl ScriptHooks {
    pub fn call_server_startup(&self, eng: split::EngineRef) {
        call_with_engine0(&self.server_startup, eng);
    }

    pub fn call_server_shutdown(&self, eng: split::EngineRef) {
        call_with_engine0(&self.server_shutdown, eng);
    }

    pub fn call_eval(&self, eng: split::EngineRef, s: &str) -> String {
        if let Some(ref func) = self.eval {
            with_engine_ref(eng, |eng| {
                call(func.borrow(), (eng, s))
            })
        } else {
            String::new()
        }
    }

    pub fn call_client_chat_command(&self,
                                    eng: split::EngineRef,
                                    cid: ClientId,
                                    msg: &str) {
        call_with_engine2(&self.client_chat_command, eng, cid, msg);
    }
}

unsafe impl GetTypeObject for ScriptHooks {
    fn get_type_object() -> PyBox {
        get_type().to_box()
    }
}


// Helper functions

pub fn call<A: Pack, R: for <'a> Unpack<'a>>(f: PyRef, args: A) -> R {
    let args = Pack::pack(args);
    let result = py::object::call(f, args.borrow(), None);
    Unpack::unpack(result.borrow())
}

pub fn call_void<A: Pack>(f: PyRef, args: A) {
    let args = Pack::pack(args);
    py::object::call(f, args.borrow(), None);
}

fn call_with_engine0(opt_func: &Option<PyBox>, eng: split::EngineRef) {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng,));
        });
    }
}

fn call_with_engine1<A>(opt_func: &Option<PyBox>,
                        eng: split::EngineRef,
                        a: A)
        where A: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a));
        });
    }
}

fn call_with_engine2<A, B>(opt_func: &Option<PyBox>,
                           eng: split::EngineRef,
                           a: A,
                           b: B)
        where A: Pack, B: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a, b));
        });
    }
}
