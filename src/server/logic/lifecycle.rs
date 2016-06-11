use std::fs::File;

use types::*;
use util::now;

use engine::Engine;
use engine::split::EngineRef;
use logic;
use messages::{ClientResponse, SyncKind};
use wire::{WireWriter, WireReader};
use world::Fragment;
use world::bundle;
use world::object::*;


pub fn start_up(mut eng: EngineRef) {
    let world_time =
        if let Some(mut file) = eng.storage().open_world_file() {
            let b = bundle::read_bundle(&mut file).unwrap();
            bundle::import_world(&mut eng.as_world_fragment(), &b);
            b.world.as_ref().unwrap().now
        } else {
            0
        };
    trace!("initial world time: {}", world_time);

    let unix_time = now();
    eng.messages_mut().set_world_time(unix_time, world_time);
    eng.timer_mut().set_world_time(unix_time, world_time);
    eng.borrow().unwrap().now = world_time;

    if let Some(mut file) = eng.storage().open_plane_file(STABLE_PLANE_LIMBO) {
        let b = bundle::read_bundle(&mut file).unwrap();
        bundle::import_bundle(&mut eng.as_hidden_world_fragment(), &b);
    } else {
        let name = "Limbo".to_owned();
        let stable_pid = eng.as_hidden_world_fragment().create_plane(name).unwrap().stable_id();
        assert!(stable_pid == STABLE_PLANE_LIMBO);
    }

    if let Some(mut file) = eng.storage().open_plane_file(STABLE_PLANE_FOREST) {
        let b = bundle::read_bundle(&mut file).unwrap();
        bundle::import_bundle(&mut eng.as_hidden_world_fragment(), &b);
    } else {
        let name = "Everfree Forest".to_owned();
        let stable_pid = eng.as_hidden_world_fragment().create_plane(name).unwrap().stable_id();
        assert!(stable_pid == STABLE_PLANE_FOREST);
    }

    warn_on_err!(eng.script_hooks().call_server_startup(eng.borrow()));
}


pub fn shut_down(mut eng: EngineRef) {
    while let Some(cid) = eng.world().clients().next().map(|c| c.id()) {
        warn_on_err!(logic::client::logout(eng.borrow().unwrap(), cid));
    }

    while let Some((pid, cpos)) = eng.world().terrain_chunks().next()
                              .map(|tc| (tc.plane_id(), tc.chunk_pos())) {
        logic::chunks::unload_chunk(eng.borrow(), pid, cpos);
    }

    while let Some(pid) = eng.world().planes().next().map(|p| p.id()) {
        logic::chunks::unload_plane(eng.borrow(), pid);
    }

    warn_on_err!(eng.script_hooks().call_server_shutdown(eng.borrow()));

    {
        let mut exporter = bundle::Exporter::new(eng.data());
        exporter.add_world(eng.world());
        let mut b = exporter.finish();
        b.world.as_mut().unwrap().now = eng.now();
        let b = b;

        let mut file = eng.storage().create_world_file();
        warn_on_err!(bundle::write_bundle(&mut file, &b));
    }
}


pub fn pre_restart(eng: EngineRef) {
    pre_restart_(eng.unwrap());
}

pub fn pre_restart_(eng: &mut Engine) {
    let msg = ClientResponse::ChatUpdate("***\tServer restarting...".to_owned());
    eng.messages.broadcast_clients(msg);
    eng.messages.broadcast_clients(ClientResponse::SyncStatus(SyncKind::Reset));

    {
        info!("recording clients to file...");
        let file = eng.storage.create_restart_file();
        let mut ww = WireWriter::new(file);
        for c in eng.world.clients() {
            let wire_id = match eng.messages.client_to_wire(c.id()) {
                Some(x) => x,
                None => {
                    warn!("no wire for client {:?}", c.id());
                    continue;
                },
            };
            let uid = match eng.extra.client_uid.get(&c.id()) {
                Some(x) => x,
                None => {
                    warn!("no user ID for client {:?}", c.id());
                    continue;
                },
            };
            ww.write_msg(wire_id, (uid, c.name())).unwrap();
        }
    }
}

pub fn post_restart(eng: EngineRef, file: File) {
    post_restart_(eng.unwrap(), file);
}

pub fn post_restart_(eng: &mut Engine, file: File) {
    info!("retrieving clients from file...");

    let mut wr = WireReader::new(file);
    while let Ok(wire_id) = wr.read_header() {
        let (uid, name) = wr.read::<(u32, String)>().unwrap();
        warn_on_err!(logic::client::login(eng, wire_id, uid, name));
    }

    let msg = ClientResponse::ChatUpdate("***\tServer restarted".to_owned());
    eng.messages.broadcast_clients(msg);
}
