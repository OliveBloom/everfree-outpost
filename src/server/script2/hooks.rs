use types::*;

use engine::split;
use python as py;
use python::{PyBox, PyRef, PyResult};

use super::{Pack, Unpack};
use super::rust_ref::{RustRef, RustRefType};
use super::engine::with_engine_ref;


macro_rules! hooks_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(ScriptHooks, $($all)*); );
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
    pub fn call_server_startup(&self, eng: split::EngineRef) -> PyResult<()> {
        call_with_engine0(&self.server_startup, eng)
    }

    pub fn call_server_shutdown(&self, eng: split::EngineRef) -> PyResult<()> {
        call_with_engine0(&self.server_shutdown, eng)
    }

    pub fn call_eval(&self, eng: split::EngineRef, s: &str) -> PyResult<String> {
        if let Some(ref func) = self.eval {
            with_engine_ref(eng, |eng| {
                call(func.borrow(), (eng, s))
            })
        } else {
            Ok(String::new())
        }
    }

    pub fn call_client_chat_command(&self,
                                    eng: split::EngineRef,
                                    cid: ClientId,
                                    msg: &str) -> PyResult<()> {
        call_with_engine2(&self.client_chat_command, eng, cid, msg)
    }
}

unsafe impl RustRefType for ScriptHooks {
    fn get_type_object() -> PyBox {
        get_type().to_box()
    }
}


// Helper functions

pub fn call<A: Pack, R: for <'a> Unpack<'a>>(f: PyRef, args: A) -> PyResult<R> {
    let args = try!(Pack::pack(args));
    let result = try!(py::object::call(f, args.borrow(), None));
    Unpack::unpack(result.borrow())
}

pub fn call_void<A: Pack>(f: PyRef, args: A) -> PyResult<()> {
    let args = try!(Pack::pack(args));
    try!(py::object::call(f, args.borrow(), None));
    Ok(())
}

fn call_with_engine0(opt_func: &Option<PyBox>, eng: split::EngineRef) -> PyResult<()> {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng,))
        })
    } else {
        Ok(())
    }
}

fn call_with_engine1<A>(opt_func: &Option<PyBox>,
                        eng: split::EngineRef,
                        a: A) -> PyResult<()>
        where A: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a))
        })
    } else {
        Ok(())
    }
}

fn call_with_engine2<A, B>(opt_func: &Option<PyBox>,
                           eng: split::EngineRef,
                           a: A,
                           b: B) -> PyResult<()>
        where A: Pack, B: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a, b))
        })
    } else {
        Ok(())
    }
}
