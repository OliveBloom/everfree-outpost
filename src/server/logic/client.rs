//! The client login flow looks like this:
//!
//! (1) A new wire (connection) appears.  Initially, the game client is not ready for any messages,
//!     so don't send anything.  Also, don't create or load a Client object yet, so the character
//!     doesn't appear to other users while the client is still in the loading screen.
//!
//! (2) The game client indicates readiness.  It's now okay to create or load the Client.
//!
//! (3) If the Client has no pawn, the player needs to complete character customization.  For
//!     vision purposes, show the area around the spawn point.
//!
//! (4) Once the client has a pawn, they are fully logged in and everything operates normally.
//!     Vision tracks the pawn as it moves around.

use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use chunks;
use engine::Engine;
use logic;
use messages::{ClientResponse, SyncKind, Dialog};
use world;
use world::Motion;
use world::bundle::{self, Bundle, Builder, AnyId};
use world::extra;
use world::fragment::DummyFragment;
use world::fragment::Fragment as World_Fragment;
use world::object::*;
use vision;


const DAY_NIGHT_CYCLE_TICKS: u32 = 24_000;
const DAY_NIGHT_CYCLE_MS: u32 = 24 * 60 * 1000;

pub fn ready(eng: &mut Engine, wire_id: WireId) -> bundle::Result<ClientId> {
    let (uid, name) = unwrap!(eng.extra.wire_info.remove(&wire_id));
    login(eng, wire_id, uid, name)
}

struct LoginPart {
    cid: ClientId,
    opt_eid: Option<EntityId>,
    pid: PlaneId,
    pos: V3,
}

pub fn login(eng: &mut Engine,
             wire_id: WireId,
             uid: u32,
             name: String) -> bundle::Result<ClientId> {
    if let Some(&old_cid) = eng.extra.uid_client.get(&uid) {
        // `kick_client` forces a logout, including saving the current position.
        eng.kick_client(old_cid, "logged in from another location");
    }

    let name = if &name == "" { format!("Anon:{:04}", uid) } else { name };
    info!("logging in: wire {:?}, uid {:x}, name {:?}", wire_id, uid, name);

    // Load or create the client bundle.
    let bundle =
        if let Some(mut file) = eng.storage.open_client_file(uid) {
            let mut b = try!(bundle::read_bundle(&mut file));
            if b.clients.len() != 1 {
                fail!("expected exactly one client in bundle");
            }
            if b.entities.len() > 1 {
                fail!("expected at most one entity in bundle");
                // Otherwise import may fail.  We ensure the plane is loaded before importing the
                // pawn, but we don't check for any other entities at the moment.
            }
            b.clients[0].name = name.clone().into_boxed_str();
            b
        } else {
            let mut b = Builder::new(eng.data);
            b.client().name(&name);
            b.finish()
        };


    let LoginPart { cid, opt_eid, pid, pos } =
        if bundle.clients[0].pawn.is_none() {
            try!(login_no_pawn(eng, bundle))
        } else {
            try!(login_with_pawn(eng, bundle))
        };
    let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));


    // Init messages and chat
    info!("{:?}: logged in as {} ({:?})", wire_id, name, cid);
    eng.messages.add_client(cid, wire_id, &name);

    let cycle_base = (eng.now % DAY_NIGHT_CYCLE_MS as Time) as u32;
    if let Some(eid) = opt_eid {
        eng.messages.send_client(cid, ClientResponse::Init(eid,
                                                           eng.now,
                                                           cycle_base,
                                                           DAY_NIGHT_CYCLE_MS));
    } else {
        eng.messages.send_client(cid, ClientResponse::InitNoPawn(pos,
                                                                 eng.now,
                                                                 cycle_base,
                                                                 DAY_NIGHT_CYCLE_MS));
    }

    eng.chat.add_client(cid, pid, cpos);

    eng.extra.client_uid.insert(cid, uid);
    eng.extra.uid_client.insert(uid, cid);

    // Init vision
    let region = vision::vision_region(pos);
    for cpos in region.points() {
        logic::chunks::load_chunk(eng.as_ref(), pid, cpos);
    }
    // TODO: figure out why add_client needs to happen after load_chunks
    // (current guess is that load_chunk is using HiddenVisionFragment)
    vision::Fragment::add_client(&mut eng.as_ref().as_vision_fragment(), cid, pid, region);

    if opt_eid.is_some() {
        // Init scripts
        warn_on_err!(eng.script_hooks.call_client_login(eng.as_ref(), cid));
    } else {
        eng.messages.send_client(cid, ClientResponse::OpenDialog(Dialog::PonyEdit(name.clone())));
    }


    // Done logging in
    eng.messages.send_client(cid, ClientResponse::SyncStatus(SyncKind::Ok));


    Ok(cid)
}


fn login_no_pawn(eng: &mut Engine, bundle: Bundle) -> bundle::Result<LoginPart> {
    // TODO: make sure login cannot fail past this point


    // Import the bundle.
    let importer = bundle::import_bundle(&mut eng.as_ref().as_world_fragment(), &bundle);
    let cid = importer.import(&ClientId(0));

    let pid = chunks::Fragment::get_plane_id(&mut eng.as_ref().as_chunks_fragment(),
                                             STABLE_PLANE_FOREST);

    Ok(LoginPart {
        cid: cid,
        opt_eid: None,
        pid: pid,
        // TODO: hardcoded spawn point
        pos: V3::new(32, 32, 0),
    })
}

fn login_with_pawn(eng: &mut Engine, bundle: Bundle) -> bundle::Result<LoginPart> {
    let c = &bundle.clients[0];
    let b_eid = unwrap!(c.pawn, "client with no pawn is not yet supported");
    let e = unwrap!(bundle.entities.get(b_eid.unwrap() as usize));


    // TODO: make sure login cannot fail past this point


    // Import the bundle, but first import the plane so that entity import will succeed.
    let pid = chunks::Fragment::get_plane_id(&mut eng.as_ref().as_chunks_fragment(),
                                             e.stable_plane);

    let importer = bundle::import_bundle(&mut eng.as_ref().as_world_fragment(), &bundle);
    let cid = importer.import(&ClientId(0));
    let eid = importer.import(&b_eid);

    // TODO: stop entity motions on logout
    let pos = eng.world.entity(eid).pos(eng.now);

    // Run handler logic
    logic::handle::entity_create(eng, eid);

    Ok(LoginPart {
        cid: cid,
        opt_eid: Some(eid),
        pid: pid,
        pos: pos,
    })
}


pub fn create_character(eng: &mut Engine, cid: ClientId, appearance: u32) -> bundle::Result<()> {
    let _ = unwrap!(eng.world.get_client(cid));


    let mut b = Builder::new(eng.data);

    let mut main_inv = None;
    let mut ability_inv = None;
    b.entity()
     .stable_plane(STABLE_PLANE_FOREST)
     .motion(Motion::stationary(V3::new(32, 32, 0), eng.now()))
     .anim("pony//stand-0")
     .appearance(appearance)
     .inventory(|i| {
        main_inv = Some(i.id());
        i.size(30);
     })
     .inventory(|i| {
        ability_inv = Some(i.id());
        i.size(30);
        if appearance & (1 << 7) != 0 {
            i.item(0, "ability/light", 1);
        }
     })
     .extra(|x| {
        let mut inv = x.set_hash("inv");
        inv.borrow().set("main", extra::Value::InventoryId(main_inv.unwrap()));
        inv.borrow().set("ability", extra::Value::InventoryId(ability_inv.unwrap()));
     });

    let bundle = b.finish();


    // Import the entity bundle.
    let importer = bundle::import_bundle(&mut eng.as_ref().as_world_fragment(), &bundle);
    let eid = importer.import(&EntityId(0));

    // Send Init before EntityAppear/MotionStart, so that the client will recognize those messages
    // as applying to its own pawn (for motion prediction purposes).
    let cycle_base = (eng.now % DAY_NIGHT_CYCLE_MS as Time) as u32;
    eng.messages.send_client(cid, ClientResponse::Init(eid,
                                                       eng.now,
                                                       cycle_base,
                                                       DAY_NIGHT_CYCLE_MS));

    warn_on_err!(DummyFragment::new(&mut eng.world).client_mut(cid).set_pawn(Some(eid)));
    logic::handle::entity_create(eng, eid);

    // Init scripts
    warn_on_err!(eng.script_hooks.call_client_login(eng.as_ref(), cid));

    Ok(())
}

pub fn logout(eng: &mut Engine, cid: ClientId) -> bundle::Result<()> {
    let uid = eng.extra.client_uid.remove(&cid).expect("no user ID for client");
    eng.extra.uid_client.remove(&uid);

    let (pid, pos) =
        if let Some(e) = eng.world.client(cid).pawn() {
            (e.plane_id(),
             e.motion().pos(eng.now))
        } else {
            (unwrap!(eng.world.transient_plane_id(STABLE_PLANE_FOREST)),
             // TODO: hardcoded spawn point
             V3::new(32, 32, 0))
        };
    let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

    // Shut down messages and chat
    eng.messages.remove_client(cid);

    eng.chat.remove_client(cid, pid, cpos);

    // Shut down vision
    vision::Fragment::remove_client(&mut eng.as_ref().as_vision_fragment(), cid);

    // Export the client bundle.
    let exporter = {
        let c = eng.world.client(cid);

        let mut exporter = bundle::Exporter::new(eng.data);
        exporter.add_client(&c);
        let b = exporter.finish();

        let mut file = eng.storage.create_client_file(uid);
        try!(bundle::write_bundle(&mut file, &b));

        exporter
    };

    // Run handler logic
    exporter.iter_exports(|id| match id {
        AnyId::Entity(eid) => logic::handle::entity_destroy(eng, eid),
        _ => {},
    });

    // Destroy the client and associated objects
    try!(world::Fragment::destroy_client(&mut eng.as_ref().as_world_fragment(), cid));

    // Now that the Entity is gone, it's safe to unload the chunks (which may trigger unloading of
    // the Plane).
    let region = vision::vision_region(pos);
    for cpos in region.points() {
        logic::chunks::unload_chunk(eng.as_ref(), pid, cpos);
    }

    Ok(())

}

pub fn update_view(eng: &mut Engine,
                   cid: ClientId,
                   old_plane: PlaneId,
                   old_cpos: V2,
                   new_plane: PlaneId,
                   new_cpos: V2) {
    let old_region = vision::vision_region_chunk(old_cpos);
    let new_region = vision::vision_region_chunk(new_cpos);
    let plane_change = new_plane != old_plane;

    for cpos in new_region.points().filter(|&p| !old_region.contains(p) || plane_change) {
        logic::chunks::load_chunk(eng.as_ref(), new_plane, cpos);
    }

    vision::Fragment::set_client_area(&mut eng.as_ref().as_vision_fragment(),
                                      cid, new_plane, new_region);

    for cpos in old_region.points().filter(|&p| !new_region.contains(p) || plane_change) {
        logic::chunks::unload_chunk(eng.as_ref(), old_plane, cpos);
    }

    eng.chat.set_client_location(cid, old_plane, old_cpos, new_plane, new_cpos);

    eng.messages.send_client(cid, ClientResponse::SyncStatus(SyncKind::Ok));
}
