use types::*;

use engine::Engine;
use engine::split2::Coded;
use logic;
use terrain_gen::Response;
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
                eng.world.destroy_terrain_chunk(tcid);
                bundle::import_bundle(&mut eng.world, &bundle)
            };

            logic::world::on_import(eng.refine(), &importer, &bundle);
        },
    }
}
