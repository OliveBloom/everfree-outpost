use types::*;

use engine::split;
use msg::ExtraArg;
use python::api as py;
use python::api::{PyBox, PyRef, PyResult};
use python::conv::{Pack, Unpack};
use python::rust_ref::{RustRef, RustRefType};
use world;

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

    timer_fired,

    client_login,
    client_chat_command,
    client_interact,
    client_use_item,
    client_use_ability,

    structure_import_hook,
    // structure_export_hook,   // unimplemented

    hack_apply_structure_extras,
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

    pub fn call_timer_fired(&self,
                            eng: split::EngineRef,
                            userdata: PyBox) -> PyResult<()> {
        call_with_engine1(&self.timer_fired, eng, userdata)
    }

    pub fn call_client_login(&self,
                             eng: split::EngineRef,
                             cid: ClientId) -> PyResult<()> {
        call_with_engine1(&self.client_login, eng, cid)
    }

    pub fn call_client_chat_command(&self,
                                    eng: split::EngineRef,
                                    cid: ClientId,
                                    msg: &str) -> PyResult<()> {
        call_with_engine2(&self.client_chat_command, eng, cid, msg)
    }

    pub fn call_client_interact(&self,
                                eng: split::EngineRef,
                                cid: ClientId,
                                args: Option<ExtraArg>) -> PyResult<()> {
        call_with_engine2(&self.client_interact, eng, cid, args)
    }

    pub fn call_client_use_item(&self,
                                eng: split::EngineRef,
                                cid: ClientId,
                                item: ItemId,
                                args: Option<ExtraArg>) -> PyResult<()> {
        call_with_engine3(&self.client_use_item, eng, cid, item, args)
    }

    pub fn call_client_use_ability(&self,
                                   eng: split::EngineRef,
                                   cid: ClientId,
                                   ability: ItemId,
                                   args: Option<ExtraArg>) -> PyResult<()> {
        call_with_engine3(&self.client_use_ability, eng, cid, ability, args)
    }


    pub fn call_structure_import_hook<'d, F>(&self,
                                             f: F,
                                             sid: StructureId) -> PyResult<()>
            where F: world::Fragment<'d> + split::Part + split::PartFlags {
        call_with_engine1(&self.structure_import_hook, f, sid)
    }


    pub fn call_hack_apply_structure_extras(&self,
                                            eng: split::EngineRef,
                                            sid: StructureId,
                                            k: &str,
                                            v: &str) -> PyResult<()> {
        call_with_engine3(&self.hack_apply_structure_extras, eng, sid, k, v)
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

fn call_with_engine0<E>(opt_func: &Option<PyBox>, eng: E) -> PyResult<()>
        where E: split::Part + split::PartFlags {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng,))
        })
    } else {
        Ok(())
    }
}

fn call_with_engine1<E, A>(opt_func: &Option<PyBox>,
                           eng: E,
                           a: A) -> PyResult<()>
        where E: split::Part + split::PartFlags,
              A: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a))
        })
    } else {
        Ok(())
    }
}

fn call_with_engine2<E, A, B>(opt_func: &Option<PyBox>,
                              eng: E,
                              a: A,
                              b: B) -> PyResult<()>
        where E: split::Part + split::PartFlags,
              A: Pack, B: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a, b))
        })
    } else {
        Ok(())
    }
}

fn call_with_engine3<E, A, B, C>(opt_func: &Option<PyBox>,
                                 eng: E,
                                 a: A,
                                 b: B,
                                 c: C) -> PyResult<()>
        where E: split::Part + split::PartFlags,
              A: Pack, B: Pack, C: Pack {
    if let Some(ref func) = *opt_func {
        with_engine_ref(eng, |eng| {
            call_void(func.borrow(), (eng, a, b, c))
        })
    } else {
        Ok(())
    }
}
