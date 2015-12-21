use python3_sys::*;

use types::*;

use engine::Engine;
use engine::glue;
use engine::split::{EngineRef, Part, PartFlags};
use logic;
use python as py;
use python::PyRef;
use script::ScriptEngine;
use world::Fragment as World_Fragment;
use world::object::*;


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
      $body:expr ) => {

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

pub fn with_engine_ref<E, F, R>(e: E, f: F) -> R
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


        // TODO: error handling for all these functions
        fn world_client_pawn_id(eng: OnlyWorld, cid: ClientId) -> Option<EntityId> {
            eng.world().client(cid).pawn_id()
        }


        fn world_entity_pos(eng: OnlyWorld, eid: EntityId) -> V3 {
            eng.world().entity(eid).pos(eng.now())
        }

        fn world_entity_plane_id(eng: OnlyWorld, eid: EntityId) -> PlaneId {
            eng.world().entity(eid).plane_id()
        }

        fn world_entity_teleport(eng: glue::WorldFragment,
                                 eid: EntityId,
                                 pos: V3) -> () {
            logic::world::teleport_entity(eng, eid, pos).unwrap();
        }

        fn world_entity_teleport_plane(eng: glue::WorldFragment,
                                       eid: EntityId,
                                       pid: PlaneId,
                                       pos: V3) -> () {
            logic::world::teleport_entity_plane(eng, eid, pid, pos).unwrap();
        }

        fn world_entity_teleport_stable_plane(eng: glue::WorldFragment,
                                              eid: EntityId,
                                              stable_pid: Stable<PlaneId>,
                                              pos: V3) -> () {
            logic::world::teleport_entity_stable_plane(eng, eid, stable_pid, pos).unwrap();
        }


        fn world_plane_stable_id(eng: glue::WorldFragment, pid: PlaneId) -> Stable<PlaneId> {
            let mut eng = eng;
            eng.plane_mut(pid).stable_id()
        }

        fn world_plane_name(eng: OnlyWorld, pid: PlaneId) -> String {
            eng.world().plane(pid).name().to_owned()
        }
    }
}
