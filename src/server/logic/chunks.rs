use std::mem;

use types::*;

use chunks;
use engine::Engine;
use engine::glue::*;
use engine::split::EngineRef;
use logic;
use terrain_gen::Fragment as TerrainGen_Fragment;
use world;
use world::Fragment as World_Fragment;
use world::bundle;
use world::flags;
use world::object::*;


pub fn load_chunk(mut eng: EngineRef, pid: PlaneId, cpos: V2) {
    trace!("load_chunk({:?}, {:?})", pid, cpos);
    chunks::Fragment::load(&mut eng.as_chunks_fragment(), pid, cpos);
}

pub fn unload_chunk(mut eng: EngineRef, pid: PlaneId, cpos: V2) {
    trace!("unload_chunk({:?}, {:?})", pid, cpos);
    chunks::Fragment::unload(&mut eng.as_chunks_fragment(), pid, cpos);
}

// NB: This should only be used when there is reason to believe none of the plane's chunks are
// loaded.
pub fn unload_plane(mut eng: EngineRef, pid: PlaneId) {
    trace!("unload_plane({:?})", pid);
    chunks::Fragment::unload_plane(&mut eng.as_chunks_fragment(), pid);
}


impl<'a, 'd> chunks::Provider for ChunkProvider<'a, 'd> {
    type E = bundle::Error;

    fn load_plane(&mut self, stable_pid: Stable<PlaneId>) -> bundle::Result<()> {
        trace!("Provider::load_plane({:?})", stable_pid);
        let mut file = unwrap!(self.storage().open_plane_file(stable_pid));
        let b = try!(bundle::read_bundle(&mut file));
        let importer = bundle::import_bundle(&mut self.as_hidden_world_fragment(), &b);

        {
            // FIXME
            let eng = unsafe { mem::transmute_copy(self) };
            logic::world::on_import(eng, &importer);
        }

        Ok(())
    }

    fn unload_plane(&mut self, pid: PlaneId) -> bundle::Result<()> {
        let stable_pid = self.as_hidden_world_fragment().plane_mut(pid).stable_id();
        trace!("Provider::unload_plane({:?})", pid);
        let exporter = {
            let p = self.world().plane(pid);

            let mut exporter = bundle::Exporter::new(self.data());
            exporter.add_plane(&p);
            let b = exporter.finish();

            let mut file = self.storage().create_plane_file(stable_pid);
            try!(bundle::write_bundle(&mut file, &b));

            exporter
        };

        {
            // FIXME
            let eng = unsafe { mem::transmute_copy(self) };
            logic::world::on_export(eng, &exporter);
        }

        try!(world::Fragment::destroy_plane(&mut self.as_hidden_world_fragment(), pid));
        Ok(())
    }

    fn load_terrain_chunk(&mut self, pid: PlaneId, cpos: V2) -> bundle::Result<()> {
        // TODO(plane): use pid + cpos for filename (to avoid requiring a stable_id)
        let opt_tcid = self.world().plane(pid).get_saved_terrain_chunk_id(cpos);
        let opt_file = opt_tcid.and_then(|tcid| self.storage().open_terrain_chunk_file(tcid));
        if let Some(mut file) = opt_file {
            trace!("Provider::load_terrain_chunk({:?}, {:?}): from file", pid, cpos);
            let b = try!(bundle::read_bundle(&mut file));
            let importer = bundle::import_bundle(&mut self.as_hidden_world_fragment(), &b);
            // TODO: do something intelligent if loading fails, so the whole server doesn't crash

            {
                // FIXME
                let eng = unsafe { mem::transmute_copy(self) };
                logic::world::on_import(eng, &importer);
            }
        } else {
            trace!("Provider::load_terrain_chunk({:?}, {:?}): from terrain_gen", pid, cpos);
            try!(self.as_terrain_gen_fragment().generate(pid, cpos));
        }
        Ok(())
    }

    fn unload_terrain_chunk(&mut self, pid: PlaneId, cpos: V2) -> bundle::Result<()> {
        // TODO(plane): use pid + cpos for filename
        trace!("Provider::unload_terrain_chunk({:?}, {:?})", pid, cpos);
        let stable_tcid = self.as_hidden_world_fragment().plane_mut(pid).save_terrain_chunk(cpos);
        let (tcid, exporter) = {
            let p = self.world().plane(pid);
            let tc = p.terrain_chunk(cpos);

            // Don't save chunks that are not fully generated, since they are filled with
            // 'placeholder' block instead of real data.  Instead, let the generated data be
            // discarded, and let the chunk be regenerated the next time it is needed.
            let mut exporter = bundle::Exporter::new(self.data());
            if !tc.flags().contains(flags::TC_GENERATION_PENDING) {
                exporter.add_terrain_chunk(&tc);
                let b = exporter.finish();

                let mut file = self.storage().create_terrain_chunk_file(stable_tcid);
                try!(bundle::write_bundle(&mut file, &b));
            }

            (tc.id(), exporter)
        };

        {
            // FIXME
            let eng = unsafe { mem::transmute_copy(self) };
            logic::world::on_export(eng, &exporter);
        }

        try!(world::Fragment::destroy_terrain_chunk(&mut self.as_hidden_world_fragment(), tcid));
        Ok(())
    }
}
