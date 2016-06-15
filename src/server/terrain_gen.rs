//! Terrain generation.  This system is actually an interface to `libterrain_gen`, which contains
//! the real terrain generation logic.
//!
//! Terrain generation can be slow (>30ms), so it always happens in the background on a worker
//! thread.  When a caller requests that a chunk be generated, this system sends a request to the
//! worker thread and returns immediately with a blank `TerrainChunk`.  When the worker thread
//! finishes generating that chunk, the system replaces the blank `TerrainChunk` with the final
//! version.
//!
//! In the overall architecture, the `TerrainGen` system is used to implement part of the
//! `chunks::Provider`, which is responsible for loading or generating new chunks.  It also
//! interfaces with the main `Enigne` loop so that "terrain gen finished" messages can be handled
//! immediately.
use std::mem;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};

use libphysics::CHUNK_SIZE;
use libterrain_gen::worker;
use types::*;
use util::StrResult;

use data::Data;
use engine::split::EngineRef;
use logic;
use storage::Storage;
use world::Fragment as World_Fragment;
use world::Hooks;
use world::StructureAttachment;
use world::flags;
use world::object::*;

pub type TerrainGenEvent = worker::Response;

pub struct TerrainGen {
    send: Sender<worker::Command>,
    recv: Receiver<worker::Response>,
    thread: JoinHandle<()>,
}

impl Drop for TerrainGen {
    fn drop(&mut self) {
        // Drop the command sender so the worker thread will shut down.
        unsafe {
            mem::replace(&mut self.send, mem::dropped());
        }

        let thread = unsafe { mem::replace(&mut self.thread, mem::dropped()) };
        match thread.join() {
            Ok(()) => {},
            Err(_) => {
                error!("failed to join terrain_gen thread on shutdown");
            },
        }
    }
}

impl TerrainGen {
    #[allow(deprecated)]    // for thread::scoped
    pub fn new(data: &Data, storage: &Storage) -> TerrainGen {
        let (send_cmd, recv_cmd) = mpsc::channel();
        let (send_result, recv_result) = mpsc::channel();

        let thread = unsafe {
            let ctx = mem::transmute((data, storage));
            thread::spawn(move || {
                let (data, storage) = ctx;
                worker::run(data, storage, recv_cmd, send_result);
            })
        };

        TerrainGen {
            send: send_cmd,
            recv: recv_result,
            thread: thread,
        }
    }

    pub fn receiver(&self) -> &Receiver<TerrainGenEvent> {
        &self.recv
    }
}

pub trait Fragment<'d>: Sized {
    fn terrain_gen_mut(&mut self) -> &mut TerrainGen;

    type WF: World_Fragment<'d>;
    fn with_world<F, R>(&mut self, f: F) -> R
            where F: FnOnce(&mut Self::WF) -> R;

    fn generate(&mut self,
                pid: PlaneId,
                cpos: V2) -> StrResult<TerrainChunkId> {
        let stable_pid = self.with_world(|wf| wf.plane_mut(pid).stable_id());
        self.terrain_gen_mut().send.send(worker::Command::Generate(stable_pid, cpos)).unwrap();
        self.with_world(move |wf| { wf.create_terrain_chunk(pid, cpos).map(|tc| tc.id()) })
    }

    fn process(&mut self, evt: TerrainGenEvent) {
        // FIXME
        let eng2: &mut logic::structure::PartialEngine = unsafe { mem::transmute_copy(self) };
        let (stable_pid, cpos, gc) = evt;
        self.with_world(move |wf| {
            let pid = unwrap_or!(wf.world().transient_plane_id(stable_pid));

            let tcid = {
                let mut p = wf.plane_mut(pid);
                let mut tc = unwrap_or!(p.get_terrain_chunk_mut(cpos));
 
                if !tc.flags().contains(flags::TC_GENERATION_PENDING) {
                    // Prevent this:
                    //  1) Load chunk, start generating
                    //  2) Unload chunk (but keep generating from #1)
                    //  3) Load chunk, start generating (queued, #1 is still going)
                    //  4) Generation #1 finishes; chunk is loaded so set its contents
                    //  5) Player modifies chunk
                    //  6) Generation #3 finishes; RESET chunk contents (erasing modifications)
                    return;
                }

                *tc.blocks_mut() = *gc.blocks;
                tc.flags_mut().remove(flags::TC_GENERATION_PENDING);
                tc.id()
            };
            wf.with_hooks(|h| h.on_terrain_chunk_update(tcid));

            let base = cpos.extend(0) * scalar(CHUNK_SIZE);
            for gs in &gc.structures {
                let sid = match wf.create_structure_unchecked(pid,
                                                              base + gs.pos,
                                                              gs.template) {
                    Ok(mut s) => {
                        warn_on_err!(s.set_attachment(StructureAttachment::Chunk));
                        s.id()
                    },
                    Err(e) => {
                        warn!("error placing generated structure: {}",
                              ::std::error::Error::description(&e));
                        continue;
                    },
                };
                unsafe {
                    use std::ptr;
                    // TODO: SUPER UNSAFE!!!
                    let ptr = wf as *mut Self::WF as *mut EngineRef;
                    let mut eng = ptr::read(ptr);

                    let sh = eng.script_hooks();
                    for (k, v) in &gs.extra {
                        warn_on_err!(sh.call_hack_apply_structure_extras(eng.borrow(), sid, k, v));
                    }
                }

                // FIXME: hack - shouldn't talk to logic from here
                logic::structure::on_create(&mut *eng2, sid);
            }
        });
    }

}
