use types::*;

use engine::Engine;
use engine::split2::Coded;
use logic;
use terrain_gen2::Response;
use world::bundle::{self, AnyId};
use world::fragment::{Fragment as World_Fragment, DummyFragment};
use world::object::*;

pub fn process(eng: &mut Engine, resp: Response) {
    match resp {
        Response::NewPlane(stable_pid, _bundle) => {
            let pid = unwrap_or!(eng.world.transient_plane_id(stable_pid));

            unimplemented!();
            // TODO: add plane flags, check that plane is safe to overwrite
            // TODO: import/merge bundle with existing plane, without dropping loaded_chunks
        },

        Response::NewChunk(stable_pid, cpos, mut bundle) => {
            let pid = unwrap_or!(eng.world.transient_plane_id(stable_pid));
            let tcid = unwrap_or!(eng.world.get_chunk(pid, cpos)).id();

            // Delete the old chunk and then import the new one.  This is easier than trying to
            // merge the old and new ones.
            logic::terrain_chunk::on_destroy(eng.refine(), tcid);
            let importer = {
                let mut wf = DummyFragment::new(&mut eng.world);
                wf.destroy_terrain_chunk(tcid);
                bundle::import_bundle(&mut wf, &bundle)
            };

            let orig_tcid = tcid;
            importer.iter_imports(|id| match id {
                AnyId::Client(_cid) =>
                    panic!("shouldn't be clients in TerrainChunk bundle"),
                AnyId::Entity(_eid) =>
                    panic!("shouldn't be entities in TerrainChunk bundle"),
                AnyId::Inventory(_iid) => {},    // nothing to do
                AnyId::Plane(_pid) =>
                    panic!("shouldn't be planes in TerrainChunk bundle"),
                AnyId::TerrainChunk(tcid) =>
                    logic::terrain_chunk::on_create(eng.refine(), tcid),
                AnyId::Structure(sid) =>
                    logic::structure::on_create(eng.refine(), sid),
            });
        },
    }
}
