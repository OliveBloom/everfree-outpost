//! Terrain generation.  This system wraps interaction with the `generate_terrain` binary, which
//! contains the real terrain generation logic.
//!
//! Terrain generation can be slow (>30ms), so it always happens in the background, with a worker
//! thread waiting for the replies.  When a caller requests that a chunk be generated, this system
//! sends a request to the worker thread and returns immediately with a blank `TerrainChunk`.  When
//! the worker thread finishes generating that chunk, the system replaces the blank `TerrainChunk`
//! with the final version.
//!
//! In the overall architecture, the `TerrainGen` system is used to implement the part of
//! `logic::chunks` that's responsible for loading or generating new chunks.  The response messages
//! are dispatched by the main `Engine` loop to `logic::terrain_gen`, which imports the newly
//! generated chunk into the `World`.
use std::io::{self, Read};
use std::mem;
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};

use libphysics::CHUNK_SIZE;
use libterrain_gen::worker;
use types::*;
use util::StrResult;
use util::bytes::{ReadBytes, WriteBytes};

use data::Data;
use engine::Engine;
use engine::split::EngineRef;
use engine::split2::Coded;
use logic;
use storage::Storage;
use world::Fragment as World_Fragment;
use world::Hooks;
use world::StructureAttachment;
use world::bundle::Bundle;
use world::bundle::flat::FlatView;
use world::flags;
use world::object::*;


enum Request {
    InitPlane(Stable<PlaneId>, u32),
    ForgetPlane(Stable<PlaneId>),
    GenPlane(Stable<PlaneId>),
    GenChunk(Stable<PlaneId>, V2),
}

const OP_INIT_PLANE: u32 =      0;
const OP_FORGET_PLANE: u32 =    1;
const OP_GEN_PLANE: u32 =       2;
const OP_GEN_CHUNK: u32 =       3;

pub enum Response {
    NewPlane(Stable<PlaneId>, Box<Bundle>),
    NewChunk(Stable<PlaneId>, V2, Box<Bundle>),
}

pub type TerrainGenEvent = Response;


pub struct TerrainGen {
    send: Sender<Request>,
    recv: Receiver<Response>,
    io_thread: JoinHandle<()>,
    subprocess: Child,
}

impl Drop for TerrainGen {
    fn drop(&mut self) {
        // Kill the child process
        warn_on_err!(self.subprocess.kill());

        // Drop the command/response channels so the worker thread will shut down.
        unsafe {
            mem::replace(&mut self.send, mem::dropped());
            mem::replace(&mut self.recv, mem::dropped());
        }

        let io_thread = unsafe { mem::replace(&mut self.io_thread, mem::dropped()) };
        // Note: can't use warn_on_err! because the error may not actually implement Error.
        match io_thread.join() {
            Ok(()) => {},
            Err(_) => { error!("failed to join terrain_gen thread on shutdown"); },
        }
    }
}

impl TerrainGen {
    pub fn new(data: &Data, storage: &Storage) -> TerrainGen {
        let (send_req, recv_req) = mpsc::channel();
        let (send_resp, recv_resp) = mpsc::channel();

        // TODO: make this smarter about finding the binary and the storage dir
        let mut child = Command::new("bin/generate_terrain").arg(".")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn generate_terrain: {}", e));

        let to_child = child.stdin.take().unwrap();
        let from_child = child.stdout.take().unwrap();
                
        let thread = unsafe {
            let ctx = mem::transmute((data, storage));
            thread::spawn(move || {
                let (data, storage) = ctx;
                io_worker(data, storage, recv_req, send_resp, to_child, from_child)
                    .unwrap_or_else(|e| panic!("io_worker failed: {}", e));
            })
        };

        TerrainGen {
            send: send_req,
            recv: recv_resp,
            io_thread: thread,
            subprocess: child,
        }
    }

    pub fn generate_chunk(&mut self, stable_pid: Stable<PlaneId>, cpos: V2) {
        self.send.send(Request::GenChunk(stable_pid, cpos))
            .expect("error sending to terrain_gen worker");
    }

    pub fn receiver(&self) -> &Receiver<Response> {
        &self.recv
    }
}


fn io_worker(data: &Data,
             storage: &Storage,
             recv: Receiver<Request>,
             send: Sender<Response>,
             mut to_child: ChildStdin,
             mut from_child: ChildStdout) -> io::Result<()> {
    for cmd in recv.iter() {
        match cmd {
            Request::InitPlane(pid, flags) => {
                try!(to_child.write_bytes(OP_INIT_PLANE));
                try!(to_child.write_bytes((pid, flags)));
                // No response expected
            },
            Request::ForgetPlane(pid) => {
                try!(to_child.write_bytes(OP_FORGET_PLANE));
                try!(to_child.write_bytes(pid));
                // No response expected
            },

            Request::GenPlane(pid) => {
                try!(to_child.write_bytes(OP_GEN_PLANE));
                try!(to_child.write_bytes(pid));
                // No response expected
                let b = try!(read_bundle(&mut from_child));
                send.send(Response::NewPlane(pid, b));
            },

            Request::GenChunk(pid, cpos) => {
                try!(to_child.write_bytes(OP_GEN_CHUNK));
                try!(to_child.write_bytes((pid, cpos)));
                let b = try!(read_bundle(&mut from_child));
                send.send(Response::NewChunk(pid, cpos, b));
            },
        }
    }
    Ok(())
}

fn read_bundle<R: Read>(r: &mut R) -> io::Result<Box<Bundle>> {
    let len = try!(r.read_bytes::<u32>()) as usize;

    let mut buf = Vec::with_capacity(len);
    unsafe {
        assert!(buf.capacity() >= len);
        buf.set_len(len);
        try!(r.read_exact(&mut buf));
    }

    let f = try!(FlatView::from_bytes(&buf));
    let b = Box::new(f.unflatten_bundle());
    Ok(b)
}
