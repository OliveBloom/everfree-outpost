#![crate_name = "backend"]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

#![feature(
    box_syntax,
    btree_range,
    collections,
    collections_bound,
    filling_drop,
    fnbox,
    mpsc_select,
    plugin,
    trace_macros,
    unboxed_closures,
    unsize,
)]
#![cfg_attr(test, feature(test))]

#![plugin(syntax_exts)]

#[macro_use] extern crate bitflags;
extern crate core;
extern crate env_logger;
extern crate libc;
#[macro_use] extern crate log;
extern crate rand;
extern crate rustc_serialize;
#[cfg(test)] extern crate test;
extern crate time;

extern crate python3_sys;

extern crate common as libcommon;
extern crate common_movement as libcommon_movement;
extern crate common_proto as libcommon_proto;
#[macro_use] extern crate common_util as libcommon_util;
extern crate physics as libphysics;
#[macro_use] extern crate python as libpython;
extern crate terrain_gen as libterrain_gen;
extern crate server_config as libserver_config;
extern crate server_bundle as libserver_bundle;
pub extern crate server_extra as libserver_extra;
extern crate server_types as libserver_types;
extern crate server_world_types as libserver_world_types;

use std::fs::File;
use std::io::{self, Read};
use rustc_serialize::json;


#[macro_use] mod util;
#[macro_use] mod engine;
mod python;

mod tasks;
mod timer;
mod types;
mod input;
mod world;
mod pubsub;
mod chat;
mod timing;

mod messages;
mod chunks;
mod terrain_gen;
mod vision;
mod logic;
mod cache;
mod dialogs;
mod components;

mod script;

#[cfg(test)] mod tests;

mod data {
    pub use libserver_config::data::*;
}

mod storage {
    pub use libserver_config::storage::*;
}


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

fn main() {
    use std::env;
    use std::sync::mpsc::channel;
    use std::thread;

    env_logger::init().unwrap();

    // Initialize engine environment.
    let args = env::args().collect::<Vec<_>>();
    let storage = storage::Storage::new(&args[1]);

    if !storage.has_save_layer(storage::SaveLayer::Delta) {
        storage.create_save_layer(storage::SaveLayer::Delta);
    }

    let mut raw_bytes = Vec::new();
    storage.open_binary_data().read_to_end(&mut raw_bytes).unwrap();
    let data = data::Data::new(raw_bytes.into_boxed_slice());

    script::ffi_module_preinit();
    python::api::initialize();
    script::ffi_module_postinit();
    // Python init failures turn into panics, since they are likely to leave the Python context in
    // an invalid state.
    python::util::run_file(&storage.script_dir().join("boot.py")).unwrap();


    // Start background threads.
    let (req_send, req_recv) = channel();
    let (resp_send, resp_recv) = channel();

    thread::spawn(move || {
        let reader = io::stdin();
        tasks::run_input(reader, req_send).unwrap();
    });

    thread::spawn(move || {
        let writer = io::BufWriter::new(io::stdout());
        tasks::run_output(writer, resp_recv).unwrap();
    });


    // Run the engine.  The engine runs inside the two `with_ref`s so that the data and storage
    // refs will be valid for the lifetime of the server.
    script::with_ref(&storage, |storage_ref| {
        script::with_ref(&data, |data_ref| {
            let mut script_hooks = script::ScriptHooks::new();
            script::with_ref_mut(&mut script_hooks, |hooks_ref| {
                script::call_init(storage_ref, data_ref, hooks_ref).unwrap();
            });
            let script_hooks = script_hooks;

            let mut engine = engine::Engine::new(&data,
                                                 &storage,
                                                 &script_hooks,
                                                 req_recv,
                                                 resp_send);
            engine.run();
        });
    });
}
