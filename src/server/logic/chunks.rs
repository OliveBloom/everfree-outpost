use std::mem;

use types::*;

use chunks;
use engine::Engine;
use engine::glue::*;
use engine::split::EngineRef;
use engine::split2::Coded;
use logic;
use terrain_gen::Fragment as TerrainGen_Fragment;
use world;
use world::fragment::{Fragment as World_Fragment, DummyFragment};
use world::bundle;
use world::flags;
use world::object::*;


pub fn get_plane_id(eng: &mut Engine, stable_pid: Stable<PlaneId>) -> PlaneId {
    unimplemented!()
}

// NB: This should only be used when there is reason to believe none of the plane's chunks are
// loaded.
pub fn unload_plane(eng: &mut Engine, pid: PlaneId) {
    unimplemented!();
}

pub fn load_chunk(eng: &mut Engine, pid: PlaneId, cpos: V2) {
    unimplemented!();
}

pub fn unload_chunk(eng: &mut Engine, pid: PlaneId, cpos: V2) {
    unimplemented!();
}


fn import_plane(eng: &mut Engine, stable_pid: Stable<PlaneId>) -> bundle::Result<()> {
    trace!("load plane {:?}", stable_pid);

    let mut file = unwrap!(eng.storage.open_plane_file(stable_pid));
    let b = try!(bundle::read_bundle(&mut file));
    let importer = bundle::import_bundle(&mut DummyFragment::new(&mut eng.world), &b);
    logic::world::on_import(eng.refine(), &importer);

    Ok(())
}

fn export_plane(eng: &mut Engine, pid: PlaneId) -> bundle::Result<()> {
    let stable_pid = DummyFragment::new(&mut eng.world).plane_mut(pid).stable_id();
    trace!("unload plane {:?}", stable_pid);

    let exporter = {
        let p = eng.world.plane(pid);

        let mut exporter = bundle::Exporter::new(eng.data);
        exporter.add_plane(&p);
        let b = exporter.finish();

        let mut file = eng.storage.create_plane_file(stable_pid);
        try!(bundle::write_bundle(&mut file, &b));

        exporter
    };

    logic::world::on_export(eng.refine(), &exporter);
    try!(DummyFragment::new(&mut eng.world).destroy_plane(pid));
    Ok(())
}


fn import_terrain_chunk(eng: &mut Engine, pid: PlaneId, cpos: V2) -> bundle::Result<()> {
    // TODO(plane): use pid + cpos for filename (to avoid requiring a stable_id)
    let opt_tcid = eng.world.plane(pid).get_saved_terrain_chunk_id(cpos);
    let opt_file = opt_tcid.and_then(|tcid| eng.storage.open_terrain_chunk_file(tcid));
    if let Some(mut file) = opt_file {
        trace!("load chunk from file: {:?} @ ({:?}, {:?})", opt_tcid, pid, cpos);
        // TODO: do something intelligent if loading fails, so the whole server doesn't crash
        let b = try!(bundle::read_bundle(&mut file));
        let importer = bundle::import_bundle(&mut DummyFragment::new(&mut eng.world), &b);
        logic::world::on_import(eng.refine(), &importer);
    } else {
        trace!("load chunk from terrain_gen: ({:?}, {:?})", pid, cpos);
        try!(eng.as_ref().as_terrain_gen_fragment().generate(pid, cpos));
    }
    Ok(())
}

fn export_terrain_chunk(eng: &mut Engine, pid: PlaneId, cpos: V2) -> bundle::Result<()> {
    // TODO(plane): use pid + cpos for filename
    trace!("unload chunk: ({:?}, {:?})", pid, cpos);
    let stable_tcid = DummyFragment::new(&mut eng.world).plane_mut(pid).save_terrain_chunk(cpos);
    let (tcid, exporter) = {
        let p = eng.world.plane(pid);
        let tc = p.terrain_chunk(cpos);

        // Don't save chunks that are not fully generated, since they are filled with
        // 'placeholder' block instead of real data.  Instead, let the generated data be
        // discarded, and let the chunk be regenerated the next time it is needed.
        let mut exporter = bundle::Exporter::new(eng.data);
        if !tc.flags().contains(flags::TC_GENERATION_PENDING) {
            exporter.add_terrain_chunk(&tc);
            let b = exporter.finish();

            let mut file = eng.storage.create_terrain_chunk_file(stable_tcid);
            try!(bundle::write_bundle(&mut file, &b));
        }

        (tc.id(), exporter)
    };

    logic::world::on_export(eng.refine(), &exporter);
    try!(DummyFragment::new(&mut eng.world).destroy_terrain_chunk(tcid));
    Ok(())
}
