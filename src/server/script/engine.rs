use std::mem;
use std::ptr;
use python3_sys::*;

use libphysics::CHUNK_SIZE;

use types::*;

use engine::Engine;
use engine::glue;
use engine::split::{EngineRef, Part, PartFlags};
use engine::split2::Coded;
use logic;
use msg::ExtraArg;
use python::api as py;
use python::api::{PyBox, PyRef, PyResult};
use python::conv::{Pack, Unpack};
use timer;
use world::{EntityAttachment, InventoryAttachment, StructureAttachment};
use world::Activity;
use world::extra::{Extra, Value, ViewMut, ArrayViewMut, HashViewMut};
use world::Fragment as World_Fragment;
use world::object::*;


pub fn init(module: PyRef) {
    init_engine(module);
    init_extra(module);
    init_extra_hash(module);
    init_extra_array(module);
}


/// Common base structure for allowing Python to obtain references to fields of objects.
struct NestedRefBase {
    base: PyObject,
    /// The reference from which this reference was derived.
    parent: *mut PyObject,
    /// The version of the parent at the time this reference was created.  If
    /// `parent.version != self.parent_ver`, then this reference has been invalidated by some
    /// subsequent mutation to the parent object.
    parent_ver: u32,
    /// The version of this reference.
    version: u32,
}

impl NestedRefBase {
    fn incr_version(&mut self) {
        let (x, overflow) = self.version.overflowing_add(1);
        assert!(!overflow);
        self.version = x;
    }

    fn valid(&self) -> bool {
        if self.parent.is_null() {
            return true;
        }

        unsafe {
            let parent = &*(self.parent as *const NestedRefBase);
            self.parent_ver == parent.version && parent.valid()
        }
    }

    /// `parent`'s representation must begin with `NestedRefBase`.
    unsafe fn set_parent(&mut self, parent: PyRef) {
        assert!(self.parent.is_null());
        self.parent = parent.to_box().unwrap();
        self.parent_ver = (*(self.parent as *mut NestedRefBase)).version;
    }

    fn dealloc(&mut self) {
        if !self.parent.is_null() {
            unsafe {
                Py_DECREF(self.parent);
                self.parent = ptr::null_mut();
            }
        }
    }
}


/// A reference to an `Engine`.  Similar to `RustRef`, but keeps different metadata (engine part
/// flags, instead of mutability).
struct PyEngineRef {
    base: NestedRefBase,
    ptr: *mut Engine<'static>,
    flags: usize,
}

macro_rules! engine_ref_func_wrapper {
    ( $wrap:ident, $engty:ty,
      $slf:ident, $args:ident, $engname:ident, $call:expr) => {
        fn $wrap(slf: $crate::python::ptr::PyRef,
                 args: $crate::python::ptr::PyRef)
                 -> $crate::python::exc::PyResult<$crate::python::ptr::PyBox> {
            use $crate::engine::split::{Part, PartFlags};
            use $crate::python::api as py;
            use $crate::script::{Pack, Unpack};

            pyassert!(py::object::is_instance(slf, get_engine_type()),
                      type_error, "expected EngineRef");
            unsafe {
                let er = &mut *(slf.as_ptr() as *mut PyEngineRef);
                let ref_flags = er.flags;
                let target_flags = <$engty as PartFlags>::flags();
                pyassert!(ref_flags & target_flags == target_flags,
                          type_error,
                          concat!("EngineRef does not have sufficient permissions for ",
                                  stringify!($engty)));
                pyassert!(er.base.valid(), runtime_error, "EngineRef has expired");

                er.flags = 0;
                er.base.incr_version();
                let result = Unpack::unpack(args)
                    .and_then(|$args| {
                        let $slf = slf;
                        let $engname = <$engty as Part>::from_ptr(er.ptr);
                        let result = $call;
                        Pack::pack(result)
                    });
                // Need to reset flags regardless of outcome
                er.flags = ref_flags;

                result
            }
        }
    };
}

macro_rules! engine_ref_func {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, $($args_rest:tt)* ),
      $ret_ty:ty,
      $body:expr ) => {

        #[allow(unused_variables)]
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, ($aname1: $aty1, $($args_rest)*), $ret_ty, $body);

            engine_ref_func_wrapper!(wrap, $aty1, slf, args, engine,
                                     imp(engine, args));
            call_wrapper!(wrap, slf, args)
        }
    };
}

macro_rules! engine_ref_func_with_ref {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, $($args_rest:tt)* ),
      $ret_ty:ty,
      $body:expr ) => {

        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp2!(imp, ($aname1: $aty1, $($args_rest)*), $ret_ty, $body);

            engine_ref_func_wrapper!(wrap, $aty1, slf, args, engine,
                                     imp(engine, slf, args));
            call_wrapper!(wrap, slf, args)
        }
    };
}

pub fn with_engine_ref<E, F, R>(e: E, f: F) -> R
        where E: Part + PartFlags, F: FnOnce(PyRef) -> R {
    unsafe {
        let obj = py::type_::instantiate(get_engine_type()).unwrap();
        {
            let er = &mut *(obj.as_ptr() as *mut PyEngineRef);
            er.ptr = e.as_ptr();
            er.flags = <E as PartFlags>::flags();
        }

        let result = f(obj.borrow());

        {
            let er = &mut *(obj.as_ptr() as *mut PyEngineRef);
            er.flags = 0;
            // Invalidate all ExtraRefs derived from `er`.
            er.base.incr_version();
        }

        result
    }
}

engine_part_typedef!(OnlyWorld(world));
engine_part_typedef!(OnlyMessages(messages));
engine_part_typedef!(OnlyTimer(timer));
engine_part_typedef!(EmptyPart());


define_python_class! {
    class EngineRef: PyEngineRef {
        type_obj ENGINE_REF_TYPE;
        initializer init_engine;
        accessor get_engine_type;
        method_macro engine_ref_func!;

    members:
    slots:
    methods:

        fn now(eng: EmptyPart,) -> Time {
            eng.now()
        }


        fn messages_clients_len(eng: OnlyMessages,) -> usize {
            eng.messages().clients_len()
        }

        fn messages_client_by_name(eng: OnlyMessages, name: String) -> Option<ClientId> {
            eng.messages().name_to_client(&name)
        }

        fn messages_send_chat_update(eng: OnlyMessages, cid: ClientId, msg: String) {
            use messages::ClientResponse;
            let resp = ClientResponse::ChatUpdate(msg);
            eng.messages().send_client(cid, resp);
        }

        fn messages_send_get_interact_args(eng: OnlyMessages,
                                           cid: ClientId,
                                           dialog_id: u32,
                                           args: ExtraArg) {
            use messages::ClientResponse;
            let resp = ClientResponse::GetInteractArgs(dialog_id, args);
            eng.messages().send_client(cid, resp);
        }

        fn messages_send_get_use_item_args(eng: OnlyMessages,
                                           cid: ClientId,
                                           item: ItemId,
                                           dialog_id: u32,
                                           args: ExtraArg) {
            use messages::ClientResponse;
            let resp = ClientResponse::GetUseItemArgs(item, dialog_id, args);
            eng.messages().send_client(cid, resp);
        }

        fn messages_send_get_use_ability_args(eng: OnlyMessages,
                                              cid: ClientId,
                                              ability: ItemId,
                                              dialog_id: u32,
                                              args: ExtraArg) {
            use messages::ClientResponse;
            let resp = ClientResponse::GetUseAbilityArgs(ability, dialog_id, args);
            eng.messages().send_client(cid, resp);
        }


        fn logic_set_main_inventories(eng: EngineRef,
                                      cid: ClientId,
                                      item_iid: InventoryId,
                                      ability_iid: InventoryId) {
            warn_on_err!(logic::items::set_main_inventories(eng, cid, item_iid, ability_iid));
        }

        fn logic_open_container(eng: EngineRef,
                                cid: ClientId,
                                iid1: InventoryId,
                                iid2: InventoryId) {
            warn_on_err!(logic::items::open_container(eng.unwrap(), cid, iid1, iid2));
        }

        fn logic_open_crafting(eng: EngineRef,
                               cid: ClientId,
                               sid: StructureId,
                               iid: InventoryId) {
            warn_on_err!(logic::items::open_crafting(eng.unwrap(), cid, sid, iid));
        }

        fn logic_set_cave(eng: glue::WorldFragment,
                          pid: PlaneId,
                          pos: V3) -> PyResult<bool> {
            let mut eng = eng;
            Ok(try!(logic::misc::set_cave(&mut eng, pid, pos)))
        }

        fn logic_set_interior(eng: glue::WorldFragment,
                              pid: PlaneId,
                              pos: V3,
                              base: String) {
            let mut eng = eng;
            warn_on_err!(logic::misc::set_block_interior(&mut eng, pid, pos, &base));
        }

        fn logic_clear_interior(eng: glue::WorldFragment,
                                pid: PlaneId,
                                pos: V3,
                                base: String,
                                new_center: String) -> PyResult<()> {
            let mut eng = eng;
            let new_center_id = pyunwrap!(eng.data().block_data.find_id(&new_center));
            warn_on_err!(logic::misc::clear_block_interior(&mut eng,
                                                           pid,
                                                           pos,
                                                           &base,
                                                           new_center_id));
            Ok(())
        }


        fn timer_schedule(eng: OnlyTimer,
                          when: Time,
                          userdata: PyBox) -> u32 {
            let mut eng = eng;
            let cookie = eng.timer_mut().schedule(when, move |eng| {
                let sh = eng.script_hooks();
                warn_on_err!(sh.call_timer_fired(eng, userdata));
            });
            cookie.raw()
        }

        fn timer_cancel(eng: OnlyTimer,
                        cookie: u32) {
            let mut eng = eng;
            eng.timer_mut().cancel(timer::Cookie::from_raw(cookie))
        }


        fn(engine_ref_func_with_ref!) world_extra(eng: glue::WorldFragment,
                                                  eng_ref: PyRef) -> PyResult<PyBox> {
            let mut eng = eng;
            let extra = eng.world_mut().extra_mut();
            unsafe { derive_extra_ref(extra, eng_ref) }
        }



        fn(engine_ref_func_with_ref!) world_client_extra(eng: glue::WorldFragment,
                                                         eng_ref: PyRef,
                                                         cid: ClientId) -> PyResult<PyBox> {
            let mut eng = eng;
            let mut c = pyunwrap!(eng.get_client_mut(cid),
                                  runtime_error, "no client with that ID");
            let extra = c.extra_mut();
            unsafe { derive_extra_ref(extra, eng_ref) }
        }

        fn world_client_name(eng: OnlyWorld, cid: ClientId) -> PyResult<String> {
            let c = pyunwrap!(eng.world().get_client(cid),
                              runtime_error, "no client with that ID");
            Ok(c.name().to_owned())
        }

        fn world_client_pawn_id(eng: OnlyWorld, cid: ClientId) -> PyResult<Option<EntityId>> {
            let c = pyunwrap!(eng.world().get_client(cid),
                              runtime_error, "no client with that ID");
            Ok(c.pawn_id())
        }


        fn(engine_ref_func_with_ref!) world_entity_extra(eng: glue::WorldFragment,
                                                         eng_ref: PyRef,
                                                         eid: EntityId) -> PyResult<PyBox> {
            let mut eng = eng;
            let mut e = pyunwrap!(eng.get_entity_mut(eid),
                                  runtime_error, "no entity with that ID");
            let extra = e.extra_mut();
            unsafe { derive_extra_ref(extra, eng_ref) }
        }

        fn world_entity_stable_id(eng: glue::WorldFragment,
                                  eid: EntityId) -> PyResult<Stable<EntityId>> {
            let mut eng = eng;
            let mut e = pyunwrap!(eng.get_entity_mut(eid),
                                  runtime_error, "no entity with that ID");
            Ok(e.stable_id())
        }

        fn world_entity_transient_id(eng: OnlyWorld,
                                     stable_eid: Stable<EntityId>) -> Option<EntityId> {
            eng.world().transient_entity_id(stable_eid)
        }

        fn world_entity_pos(eng: OnlyWorld, eid: EntityId) -> PyResult<V3> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            Ok(e.pos(eng.now()))
        }

        fn world_entity_facing(eng: OnlyWorld, eid: EntityId) -> PyResult<V3> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            Ok(e.facing())
        }

        fn world_entity_plane_id(eng: OnlyWorld, eid: EntityId) -> PyResult<PlaneId> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            Ok(e.plane_id())
        }

        fn world_entity_appearance(eng: OnlyWorld, eid: EntityId) -> PyResult<u32> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            Ok(e.appearance())
        }

        fn world_entity_set_appearance(eng: glue::WorldFragment,
                                       eid: EntityId,
                                       appearance: u32) -> PyResult<()> {
            // FIXME: actually-unsafe transmute
            let eng = unsafe { mem::transmute(eng) };
            let ok = logic::entity::set_appearance(eng, eid, appearance);
            // Bad eid is currently the only possible failure mode
            pyassert!(ok, runtime_error, "no entity with that ID");
            Ok(())
        }

        fn world_entity_controller(eng: OnlyWorld, eid: EntityId) -> PyResult<Option<ClientId>> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            if let EntityAttachment::Client(cid) = e.attachment() {
                let c = eng.world().client(cid);
                if c.pawn_id() == Some(eid) {
                    return Ok(Some(cid));
                }
            }
            Ok(None)
        }

        fn world_entity_teleport(eng: glue::WorldFragment,
                                 eid: EntityId,
                                 pos: V3) -> PyResult<()> {
            try!(logic::world::teleport_entity(eng, eid, pos));
            Ok(())
        }

        fn world_entity_teleport_plane(eng: glue::WorldFragment,
                                       eid: EntityId,
                                       pid: PlaneId,
                                       pos: V3) -> PyResult<()> {
            try!(logic::world::teleport_entity_plane(eng, eid, pid, pos));
            Ok(())
        }

        fn world_entity_teleport_stable_plane(eng: glue::WorldFragment,
                                              eid: EntityId,
                                              stable_pid: Stable<PlaneId>,
                                              pos: V3) -> PyResult<()> {
            try!(logic::world::teleport_entity_stable_plane(eng, eid, stable_pid, pos));
            Ok(())
        }

        fn world_entity_set_activity_walk(eng: glue::WorldFragment,
                                          eid: EntityId) -> PyResult<()> {
            // FIXME: actually-unsafe transmute
            let eng = unsafe { mem::transmute(eng) };
            let ok = logic::entity::set_activity(eng, eid, Activity::Walk);
            // Bad eid is currently the only possible failure mode
            pyassert!(ok, runtime_error, "no entity with that ID");
            Ok(())
        }

        fn world_entity_set_activity_emote(eng: glue::WorldFragment,
                                           eid: EntityId,
                                           anim: AnimId) -> PyResult<()> {
            // FIXME: actually-unsafe transmute
            let eng = unsafe { mem::transmute(eng) };
            let ok = logic::entity::set_activity(eng, eid, Activity::Emote(anim));
            // Bad eid is currently the only possible failure mode
            pyassert!(ok, runtime_error, "no entity with that ID");
            Ok(())
        }

        fn world_entity_set_activity_work(eng: glue::WorldFragment,
                                          eid: EntityId,
                                          anim: AnimId,
                                          icon: AnimId) -> PyResult<()> {
            // FIXME: actually-unsafe transmute
            let eng = unsafe { mem::transmute(eng) };
            let ok = logic::entity::set_activity(eng, eid, Activity::Work(anim, icon));
            // Bad eid is currently the only possible failure mode
            pyassert!(ok, runtime_error, "no entity with that ID");
            Ok(())
        }


        fn world_inventory_create(eng: glue::WorldFragment,
                                  size: u8) -> PyResult<InventoryId> {
            let mut eng = eng;
            let i = try!(eng.create_inventory(size));
            Ok(i.id())
        }

        fn world_inventory_attach(eng: glue::WorldFragment,
                                  iid: InventoryId,
                                  attachment: InventoryAttachment) -> PyResult<()> {
            let mut eng = eng;
            let mut i = pyunwrap!(eng.get_inventory_mut(iid),
                                  runtime_error, "no inventory with that ID");
            try!(i.set_attachment(attachment));
            Ok(())
        }

        fn world_inventory_count(eng: OnlyWorld,
                                 iid: InventoryId,
                                 item: ItemId) -> PyResult<u16> {
            let i = pyunwrap!(eng.world().get_inventory(iid),
                              runtime_error, "no inventory with that ID");
            Ok(i.count(item))
        }

        fn world_inventory_count_space(eng: OnlyWorld,
                                       iid: InventoryId,
                                       item: ItemId) -> PyResult<u16> {
            let i = pyunwrap!(eng.world().get_inventory(iid),
                              runtime_error, "no inventory with that ID");
            Ok(i.count_space(item))
        }

        fn world_inventory_bulk_add(eng: glue::WorldFragment,
                                    iid: InventoryId,
                                    item: ItemId,
                                    count: u16) -> PyResult<u16> {
            let mut eng = eng;
            let mut i = pyunwrap!(eng.get_inventory_mut(iid),
                                  runtime_error, "no inventory with that ID");
            Ok(i.bulk_add(item, count))
        }

        fn world_inventory_bulk_remove(eng: glue::WorldFragment,
                                       iid: InventoryId,
                                       item: ItemId,
                                       count: u16) -> PyResult<u16> {
            let mut eng = eng;
            let mut i = pyunwrap!(eng.get_inventory_mut(iid),
                                  runtime_error, "no inventory with that ID");
            Ok(i.bulk_remove(item, count))
        }


        fn world_plane_create(eng: glue::WorldFragment,
                              name: String) -> PyResult<PlaneId> {
            let mut eng = eng;
            let p = try!(eng.create_plane(name));
            Ok(p.id())
        }

        fn world_plane_stable_id(eng: glue::WorldFragment,
                                 pid: PlaneId) -> PyResult<Stable<PlaneId>> {
            let mut eng = eng;
            let mut p = pyunwrap!(eng.get_plane_mut(pid),
                                  runtime_error, "no plane with that ID");
            Ok(p.stable_id())
        }

        fn world_plane_transient_id(eng: OnlyWorld,
                                    stable_pid: Stable<PlaneId>) -> Option<PlaneId> {
            eng.world().transient_plane_id(stable_pid)
        }

        fn(engine_ref_func_with_ref!) world_plane_extra(eng: glue::WorldFragment,
                                                        eng_ref: PyRef,
                                                        pid: PlaneId) -> PyResult<PyBox> {
            let mut eng = eng;
            let mut p = pyunwrap!(eng.get_plane_mut(pid),
                                  runtime_error, "no plane with that ID");
            let extra = p.extra_mut();
            unsafe { derive_extra_ref(extra, eng_ref) }
        }

        fn world_plane_name(eng: OnlyWorld, pid: PlaneId) -> PyResult<String> {
            let p = pyunwrap!(eng.world().get_plane(pid),
                              runtime_error, "no plane with that ID");
            Ok(p.name().to_owned())
        }

        fn world_plane_get_block(eng: OnlyWorld, pid: PlaneId, pos: V3) -> PyResult<BlockId> {
            let p = pyunwrap!(eng.world().get_plane(pid),
                              runtime_error, "no plane with that ID");
            let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE));
            let tc = pyunwrap!(p.get_terrain_chunk(cpos),
                               runtime_error, "no terrain chunk at that position");
            let idx = tc.bounds().index(pos);
            Ok(tc.blocks()[idx])
        }


        fn world_structure_create(eng: glue::WorldFragment,
                                  pid: PlaneId,
                                  pos: V3,
                                  template_id: TemplateId) -> PyResult<StructureId> {
            let mut eng = eng;
            // FIXME hacky transmute
            let eng2: &mut logic::structure::EngineLifecycle = unsafe { mem::transmute_copy(&eng) };
            let sid = try!(logic::structure::checked_create(eng2.refine(), pid, pos, template_id));
            let mut s = eng.structure_mut(sid);
            try!(s.set_attachment(StructureAttachment::Chunk));
            logic::structure::on_create(eng2, s.id());
            Ok(s.id())
        }

        fn world_structure_destroy(eng: glue::WorldFragment,
                                   sid: StructureId) -> PyResult<()> {
            let mut eng = eng;
            // FIXME hacky transmute
            let eng2 = unsafe { mem::transmute_copy(&eng) };
            logic::structure::on_destroy_recursive(eng2, sid);
            try!(eng.destroy_structure(sid));
            Ok(())
        }

        fn world_structure_replace(eng: glue::WorldFragment,
                                   sid: StructureId,
                                   template_id: TemplateId) -> PyResult<()> {
            // FIXME hacky transmute
            let eng2: &mut logic::structure::EngineLifecycle = unsafe { mem::transmute_copy(&eng) };
            let old_template_id = {
                let s = pyunwrap!(eng2.world.get_structure(sid),
                                  runtime_error, "no structure with that ID");
                s.template_id()
            };
            try!(logic::structure::checked_replace(eng2.refine(), sid, template_id));
            logic::structure::on_replace(eng2, sid, old_template_id);
            Ok(())
        }

        fn world_structure_stable_id(eng: glue::WorldFragment,
                                     sid: StructureId) -> PyResult<Stable<StructureId>> {
            let mut eng = eng;
            let mut s = pyunwrap!(eng.get_structure_mut(sid),
                                  runtime_error, "no structure with that ID");
            Ok(s.stable_id())
        }

        fn world_structure_transient_id(eng: OnlyWorld,
                                        stable_sid: Stable<StructureId>) -> Option<StructureId> {
            eng.world().transient_structure_id(stable_sid)
        }

        fn(engine_ref_func_with_ref!) world_structure_extra(eng: glue::WorldFragment,
                                                            eng_ref: PyRef,
                                                            sid: StructureId) -> PyResult<PyBox> {
            let mut eng = eng;
            let mut s = pyunwrap!(eng.get_structure_mut(sid),
                                  runtime_error, "no structure with that ID");
            let extra = s.extra_mut();
            unsafe { derive_extra_ref(extra, eng_ref) }
        }

        fn world_structure_check(eng: OnlyWorld, sid: StructureId) -> bool {
            eng.world().get_structure(sid).is_some()
        }

        fn world_structure_pos(eng: OnlyWorld, sid: StructureId) -> PyResult<V3> {
            let s = pyunwrap!(eng.world().get_structure(sid),
                              runtime_error, "no structure with that ID");
            Ok(s.pos())
        }

        fn world_structure_plane_id(eng: OnlyWorld, sid: StructureId) -> PyResult<PlaneId> {
            let s = pyunwrap!(eng.world().get_structure(sid),
                              runtime_error, "no structure with that ID");
            Ok(s.plane_id())
        }

        fn world_structure_template_id(eng: OnlyWorld, sid: StructureId) -> PyResult<TemplateId> {
            let s = pyunwrap!(eng.world().get_structure(sid),
                              runtime_error, "no structure with that ID");
            Ok(s.template_id())
        }

        fn world_structure_set_has_import_hook(eng: glue::WorldFragment,
                                               sid: StructureId,
                                               set: bool) -> PyResult<()> {
            use world::flags;
            let mut eng = eng;
            let mut s = pyunwrap!(eng.get_structure_mut(sid),
                                  runtime_error, "no structure with that ID");
            let flags =
                if set {
                    s.flags() | flags::S_HAS_IMPORT_HOOK
                } else {
                    s.flags() & !flags::S_HAS_IMPORT_HOOK
                };
            s.set_flags(flags);
            Ok(())
        }

        fn world_structure_find_at_point(eng: OnlyWorld,
                                         pid: PlaneId,
                                         pos: V3) -> Option<StructureId> {
            let chunk = pos.reduce().div_floor(scalar(CHUNK_SIZE));
            let mut best_id = None;
            let mut best_layer = 0;
            for s in eng.world().chunk_structures(pid, chunk) {
                if s.bounds().contains(pos) {
                    if s.template().layer >= best_layer {
                        best_layer = s.template().layer;
                        best_id = Some(s.id());
                    }
                }
            }
            best_id
        }

        fn world_structure_find_at_point_layer(eng: OnlyWorld,
                                               pid: PlaneId,
                                               pos: V3,
                                               layer: u8) -> Option<StructureId> {
            let chunk = pos.reduce().div_floor(scalar(CHUNK_SIZE));
            for s in eng.world().chunk_structures(pid, chunk) {
                if s.bounds().contains(pos) && s.template().layer == layer {
                    return Some(s.id())
                }
            }
            None
        }
    }
}


/// Reference to an `Extra` value.
struct ExtraRef {
    base: NestedRefBase,
    ptr: *mut Extra,
}

struct ExtraArrayRef {
    base: NestedRefBase,
    ptr: *mut (),
}

struct ExtraHashRef {
    base: NestedRefBase,
    ptr: *mut (),
}

/// Unsafe because it loses the borrow.  It's also memory-unsafe to call this function twice
/// without incrementing the version of the parent.
unsafe fn derive_extra_ref(x: &mut Extra, parent: PyRef) -> PyResult<PyBox> {
    let obj = try!(py::type_::instantiate(get_extra_type()));
    {
        let r = &mut *(obj.as_ptr() as *mut ExtraRef);
        r.base.set_parent(parent);
        r.ptr = x as *mut _;
    }
    Ok(obj)
}

unsafe fn derive_extra_array_ref(x: ArrayViewMut, parent: PyRef) -> PyResult<PyBox> {
    let obj = try!(py::type_::instantiate(get_extra_array_type()));
    {
        let r = &mut *(obj.as_ptr() as *mut ExtraArrayRef);
        r.base.set_parent(parent);
        r.ptr = mem::transmute(x);
    }
    Ok(obj)
}

unsafe fn derive_extra_hash_ref(x: HashViewMut, parent: PyRef) -> PyResult<PyBox> {
    let obj = try!(py::type_::instantiate(get_extra_hash_type()));
    {
        let r = &mut *(obj.as_ptr() as *mut ExtraHashRef);
        r.base.set_parent(parent);
        r.ptr = mem::transmute(x);
    }
    Ok(obj)
}

trait NestedRefType {
    fn get_type() -> PyRef<'static>;
}

impl NestedRefType for ExtraRef {
    fn get_type() -> PyRef<'static> {
        get_extra_type()
    }
}

impl NestedRefType for ExtraArrayRef {
    fn get_type() -> PyRef<'static> {
        get_extra_array_type()
    }
}

impl NestedRefType for ExtraHashRef {
    fn get_type() -> PyRef<'static> {
        get_extra_hash_type()
    }
}

macro_rules! nested_ref_wrapper {
    ( $T:ty, $wrap:ident, $slf:ident, $slf_ref:ident, $args:ident; $call:expr ) => {
        fn $wrap($slf_ref: $crate::python::ptr::PyRef,
                 args: $crate::python::ptr::PyRef)
                 -> $crate::python::exc::PyResult<$crate::python::ptr::PyBox> {
            use $crate::script::{Pack, Unpack};

            pyassert!(py::object::is_instance($slf_ref, <$T as NestedRefType>::get_type()),
                      type_error,
                      concat!("expected ", stringify!($T)));
            unsafe {
                let $slf = &mut *($slf_ref.as_ptr() as *mut $T);
                pyassert!($slf.base.valid(),
                          runtime_error,
                          concat!(stringify!($T), " has expired"));

                $slf.base.incr_version();
                let $args = try!(Unpack::unpack(args));
                let result = $call;
                Pack::pack(result)
            }
        }
    };
}

macro_rules! any_extra_ref_func {
    ( $T:ty, $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, $args, $ret_ty, $body);
            nested_ref_wrapper!($T, wrap, slf, _slf_ref, args;
                                // Convert *mut Extra -> &mut Extra, or for view refs,
                                // convert *mut () -> Hash/ArrayViewMut
                                imp(mem::transmute(slf.ptr), args));
            call_wrapper!(wrap, slf, args)
        }
    };
}

macro_rules! any_extra_ref_func_with_ref {
    ( $T:ty, $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp2!(imp, $args, $ret_ty, $body);
            nested_ref_wrapper!($T, wrap, slf, slf_ref, args;
                                imp(mem::transmute(slf.ptr), slf_ref, args));
            call_wrapper!(wrap, slf, args)
        }
    };
}

macro_rules! extra_ref_func {
    ( $($args:tt)* ) => ( any_extra_ref_func!(ExtraRef, $($args)*) );
}
macro_rules! extra_ref_func_with_ref {
    ( $($args:tt)* ) => ( any_extra_ref_func_with_ref!(ExtraRef, $($args)*) );
}

macro_rules! extra_array_ref_func {
    ( $($args:tt)* ) => ( any_extra_ref_func!(ExtraArrayRef, $($args)*) );
}
macro_rules! extra_array_ref_func_with_ref {
    ( $($args:tt)* ) => ( any_extra_ref_func_with_ref!(ExtraArrayRef, $($args)*) );
}

macro_rules! extra_hash_ref_func {
    ( $($args:tt)* ) => ( any_extra_ref_func!(ExtraHashRef, $($args)*) );
}
macro_rules! extra_hash_ref_func_with_ref {
    ( $($args:tt)* ) => ( any_extra_ref_func_with_ref!(ExtraHashRef, $($args)*) );
}

unsafe fn pack_view(this_ref: PyRef, view: ViewMut) -> PyResult<PyBox> {
    match view {
        ViewMut::Value(v) => Pack::pack(v),
        ViewMut::Array(a) => derive_extra_array_ref(a, this_ref),
        ViewMut::Hash(h) => derive_extra_hash_ref(h, this_ref),
    }
}

define_python_class! {
    class ExtraRef: ExtraRef {
        type_obj EXTRA_REF_TYPE;
        initializer init_extra;
        accessor get_extra_type;
        method_macro extra_ref_func!;

        fn(raw_func!) is_valid(this: PyRef,) -> PyResult<bool> {
            pyassert!(py::object::is_instance(this, get_extra_type()),
                      type_error, "expected an ExtraRef");
            unsafe {
                let er = &mut *(this.as_ptr() as *mut ExtraRef);
                Ok(er.base.valid())
            }
        }

        fn(extra_ref_func_with_ref!) get(this: &mut Extra,
                                         this_ref: PyRef,
                                         key: String) -> PyResult<PyBox> {
            let view = pyunwrap!(this.get_mut(&key),
                                 key_error, "key not present: {:?}", key);
            unsafe { pack_view(this_ref, view) }
        }

        fn set_value(this: &mut Extra, key: String, val: Value) {
            this.set(&key, val);
        }

        fn set_array(this: &mut Extra, key: String) {
            this.set_array(&key);
        }

        fn set_hash(this: &mut Extra, key: String) {
            this.set_hash(&key);
        }

        fn remove(this: &mut Extra, key: String) {
            this.remove(&key);
        }

        fn contains(this: &mut Extra, key: String) -> bool {
            this.contains(&key)
        }

        fn len(this: &mut Extra) -> usize {
            this.len()
        }

        fn(extra_ref_func_with_ref!) convert(this: &mut Extra,
                                             this_ref: PyRef) -> PyResult<PyBox> {
            let dict = try!(py::dict::new());
            for (key, val) in this.iter_mut() {
                let py_val = try!(unsafe { pack_view(this_ref, val) });
                try!(py::dict::set_item_str(dict.borrow(), key, py_val.borrow()));
            }
            Ok(dict)
        }
    }
}

define_python_class! {
    class ExtraArrayRef: ExtraArrayRef {
        type_obj EXTRA_ARRAY_REF_TYPE;
        initializer init_extra_array;
        accessor get_extra_array_type;
        method_macro extra_array_ref_func!;

        fn(raw_func!) is_valid(this: PyRef,) -> PyResult<bool> {
            pyassert!(py::object::is_instance(this, get_extra_array_type()),
                      type_error, "expected an ExtraArrayRef");
            unsafe {
                let er = &mut *(this.as_ptr() as *mut ExtraArrayRef);
                Ok(er.base.valid())
            }
        }

        fn(extra_array_ref_func_with_ref!) get(this: ArrayViewMut,
                                               this_ref: PyRef,
                                               idx: usize) -> PyResult<PyBox> {
            let mut this = this;
            let len = this.borrow().len();
            pyassert!(idx < len,
                      index_error, "the len is {} but the index is {}", len, idx);
            let view = this.get_mut(idx);
            unsafe { pack_view(this_ref, view) }
        }

        fn set_value(this: ArrayViewMut, idx: usize, val: Value) {
            this.set(idx, val);
        }

        fn set_array(this: ArrayViewMut, idx: usize) {
            this.set_array(idx);
        }

        fn set_hash(this: ArrayViewMut, idx: usize) {
            this.set_hash(idx);
        }

        fn push(this: ArrayViewMut) {
            this.push();
        }

        fn pop(this: ArrayViewMut) -> PyResult<()> {
            let mut this = this;
            pyassert!(this.borrow().len() > 0,
                      value_error, "can't pop from empty list");
            this.pop();
            Ok(())
        }

        fn len(this: ArrayViewMut) -> usize {
            this.len()
        }

        fn(extra_array_ref_func_with_ref!) convert(this: ArrayViewMut,
                                                   this_ref: PyRef) -> PyResult<PyBox> {
            let list = try!(py::list::new());
            for val in this.iter_mut() {
                let py_val = try!(unsafe { pack_view(this_ref, val) });
                try!(py::list::append(list.borrow(), py_val.borrow()));
            }
            Ok(list)
        }
    }
}

define_python_class! {
    class ExtraHashRef: ExtraHashRef {
        type_obj EXTRA_HASH_REF_TYPE;
        initializer init_extra_hash;
        accessor get_extra_hash_type;
        method_macro extra_hash_ref_func!;

        fn(raw_func!) is_valid(this: PyRef,) -> PyResult<bool> {
            pyassert!(py::object::is_instance(this, get_extra_hash_type()),
                      type_error, "expected an ExtraHashRef");
            unsafe {
                let er = &mut *(this.as_ptr() as *mut ExtraHashRef);
                Ok(er.base.valid())
            }
        }

        fn(extra_hash_ref_func_with_ref!) get(this: HashViewMut,
                                              this_ref: PyRef,
                                              key: String) -> PyResult<PyBox> {
            let view = pyunwrap!(this.get_mut(&key),
                                 key_error, "key not present: {:?}", key);
            unsafe { pack_view(this_ref, view) }
        }

        fn set_value(this: HashViewMut, key: String, val: Value) {
            this.set(&key, val);
        }

        fn set_array(this: HashViewMut, key: String) {
            this.set_array(&key);
        }

        fn set_hash(this: HashViewMut, key: String) {
            this.set_hash(&key);
        }

        fn remove(this: HashViewMut, key: String) {
            this.remove(&key);
        }

        fn contains(this: HashViewMut, key: String) -> bool {
            this.contains(&key)
        }

        fn len(this: HashViewMut) -> usize {
            this.len()
        }

        fn(extra_hash_ref_func_with_ref!) convert(this: HashViewMut,
                                                  this_ref: PyRef) -> PyResult<PyBox> {
            let dict = try!(py::dict::new());
            for (key, val) in this.iter_mut() {
                let py_val = try!(unsafe { pack_view(this_ref, val) });
                try!(py::dict::set_item_str(dict.borrow(), key, py_val.borrow()));
            }
            Ok(dict)
        }
    }
}
