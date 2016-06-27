use server_extra::Value;

use api as py;
use api::{PyBox, PyRef, PyResult};
use conv::{Pack, Unpack};

use super::{types, v3};


impl<'a> Unpack<'a> for Value {
    fn unpack(obj: PyRef<'a>) -> PyResult<Value> {
        // IDs
        if py::object::is_instance(obj, types::get_client_id_type()) {
            Ok(Value::ClientId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_entity_id_type()) {
            Ok(Value::EntityId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_inventory_id_type()) {
            Ok(Value::InventoryId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_plane_id_type()) {
            Ok(Value::PlaneId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_terrain_chunk_id_type()) {
            Ok(Value::TerrainChunkId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_structure_id_type()) {
            Ok(Value::StructureId(try!(Unpack::unpack(obj))))

        // Stable IDs
        } else if py::object::is_instance(obj, types::get_stable_client_id_type()) {
            Ok(Value::StableClientId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_stable_entity_id_type()) {
            Ok(Value::StableEntityId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_stable_inventory_id_type()) {
            Ok(Value::StableInventoryId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_stable_plane_id_type()) {
            Ok(Value::StablePlaneId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_stable_terrain_chunk_id_type()) {
            Ok(Value::StableTerrainChunkId(try!(Unpack::unpack(obj))))
        } else if py::object::is_instance(obj, types::get_stable_structure_id_type()) {
            Ok(Value::StableStructureId(try!(Unpack::unpack(obj))))

        // Vn/Region
        } else if py::object::is_instance(obj, v3::get_v3_type()) {
            Ok(Value::V3(try!(Unpack::unpack(obj))))

        // Primitives
        } else if py::bool::check(obj) {
            Ok(Value::Bool(try!(Unpack::unpack(obj))))
        } else if py::int::check(obj) {
            Ok(Value::Int(try!(Unpack::unpack(obj))))
        } else if py::float::check(obj) {
            Ok(Value::Float(try!(Unpack::unpack(obj))))
        } else if py::unicode::check(obj) {
            Ok(Value::Str(try!(Unpack::unpack(obj))))
        } else if obj == py::none() {
            Ok(Value::Null)

        // Error case
        } else {
            pyraise!(type_error, "expected something convertible to Value");
        }
    }
}

impl Pack for Value {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            Value::Null => Ok(py::none().to_box()),
            Value::Bool(b) => Pack::pack(b),
            Value::Int(i) => Pack::pack(i),
            Value::Float(f) => Pack::pack(f),
            Value::Str(s) => Pack::pack(s),

            Value::ClientId(cid) => Pack::pack(cid),
            Value::EntityId(eid) => Pack::pack(eid),
            Value::InventoryId(iid) => Pack::pack(iid),
            Value::PlaneId(pid) => Pack::pack(pid),
            Value::TerrainChunkId(tcid) => Pack::pack(tcid),
            Value::StructureId(sid) => Pack::pack(sid),

            Value::StableClientId(cid) => Pack::pack(cid),
            Value::StableEntityId(eid) => Pack::pack(eid),
            Value::StableInventoryId(iid) => Pack::pack(iid),
            Value::StablePlaneId(pid) => Pack::pack(pid),
            Value::StableTerrainChunkId(tcid) => Pack::pack(tcid),
            Value::StableStructureId(sid) => Pack::pack(sid),

            Value::V2(_v2) => pyraise!(type_error, "V2 is not supported"),
            Value::V3(v3) => Pack::pack(v3),
            Value::Region2(_region2) => pyraise!(type_error, "Region2 is not supported"),
            Value::Region3(_region3) => pyraise!(type_error, "Region3 is not supported"),
        }
    }
}


