use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use chunks;
use engine::split::EngineRef;
use logic;
use messages::{ClientResponse, SyncKind};
use world;
use world::Motion;
use world::bundle::{self, Builder, AnyId};
use world::extra;
use world::object::*;
use vision;


const DAY_NIGHT_CYCLE_TICKS: u32 = 24_000;
const DAY_NIGHT_CYCLE_MS: u32 = 24 * 60 * 1000;

pub fn register(eng: EngineRef, name: &str, appearance: u32) -> bundle::Result<()> {
    let mut b = Builder::new(eng.data());

    b.client()
     .name(name)
     .pawn(|e| {
        let mut main_inv = None;
        let mut ability_inv = None;
        e.stable_plane(STABLE_PLANE_FOREST)
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
     });

    let b = b.finish();

    let mut file = eng.storage().create_client_file(name);
    try!(bundle::write_bundle(&mut file, &b));

    Ok(())
}

pub fn login(mut eng: EngineRef, wire_id: WireId, name: &str) -> bundle::Result<()> {
    let now = eng.now();

    if let Some(old_cid) = eng.messages().name_to_client(name) {
        // `kick_client` forces a logout, including saving the current position.
        eng.borrow().unwrap().kick_client(old_cid, "logged in from another location");
    }

    // Load the client bundle
    let mut file = unwrap!(eng.storage().open_client_file(name),
                           "client file not found");
    let b = try!(bundle::read_bundle(&mut file));

    if b.clients.len() != 1 {
        fail!("expected exactly one client in bundle");
    }
    if b.entities.len() != 1 {
        fail!("expected exactly one entity in bundle");
        // Otherwise import may fail.  We ensure the plane is loaded before importing the pawn, but
        // we don't check for any other entities at the moment.
    }
    let c = &b.clients[0];
    let b_eid = unwrap!(c.pawn, "client with no pawn is not yet supported");
    let e = unwrap!(b.entities.get(b_eid.unwrap() as usize));


    // TODO: make sure login cannot fail past this point


    // Import the bundle, but first import the plane so that entity import will succeed.
    let pid = chunks::Fragment::get_plane_id(&mut eng.as_chunks_fragment(), e.stable_plane);

    let importer = bundle::import_bundle(&mut eng.as_world_fragment(), &b);
    let cid = importer.import(&ClientId(0));
    let eid = importer.import(&b_eid);

    // TODO: stop entity motions on logout
    let pos = eng.world().entity(eid).pos(now);
    let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

    // Run handler logic
    logic::handle::entity_create(eng.borrow().unwrap(), eid);

    // Init messages and chat
    info!("{:?}: logged in as {} ({:?})", wire_id, name, cid);
    eng.messages_mut().add_client(cid, wire_id, name);

    let cycle_base = (now % DAY_NIGHT_CYCLE_MS as Time) as u32;
    eng.messages_mut().send_client(cid, ClientResponse::Init(Some(eid),
                                                             now,
                                                             cycle_base,
                                                             DAY_NIGHT_CYCLE_MS));

    eng.chat_mut().add_client(cid, pid, cpos);

    // Init vision
    let region = vision::vision_region(pos);
    for cpos in region.points() {
        logic::chunks::load_chunk(eng.borrow(), pid, cpos);
    }
    // TODO: figure out why add_client needs to happen after load_chunks
    // (current guess is that load_chunk is using HiddenVisionFragment)
    vision::Fragment::add_client(&mut eng.as_vision_fragment(), cid, pid, region);

    // Init scripts
    warn_on_err!(eng.script_hooks().call_client_login(eng.borrow(), cid));


    // Done logging in
    eng.messages().send_client(cid, ClientResponse::SyncStatus(SyncKind::Ok));
    Ok(())
}

pub fn logout(mut eng: EngineRef, cid: ClientId) -> bundle::Result<()> {
    let now = eng.now();

    let (eid, pid, pos) = {
        let w = eng.world();
        let c = w.client(cid);
        let e = c.pawn().unwrap();
        (e.id(),
         e.plane_id(),
         e.motion().pos(now))
    };
    let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

    // Shut down messages and chat
    eng.messages_mut().remove_client(cid);

    eng.chat_mut().remove_client(cid, pid, cpos);

    // Shut down vision
    vision::Fragment::remove_client(&mut eng.as_vision_fragment(), cid);

    // Export the client bundle.
    let exporter = {
        let c = eng.world().client(cid);

        let mut exporter = bundle::Exporter::new(eng.data());
        exporter.add_client(&c);
        let b = exporter.finish();

        let mut file = eng.storage().create_client_file(c.name());
        try!(bundle::write_bundle(&mut file, &b));

        exporter
    };

    // Run handler logic
    exporter.iter_exports(|id| match id {
        AnyId::Entity(eid) => logic::handle::entity_destroy(eng.borrow().unwrap(), eid),
        _ => {},
    });

    // Destroy the client and associated objects
    try!(world::Fragment::destroy_client(&mut eng.as_world_fragment(), cid));

    // Now that the Entity is gone, it's safe to unload the chunks (which may trigger unloading of
    // the Plane).
    let region = vision::vision_region(pos);
    for cpos in region.points() {
        logic::chunks::unload_chunk(eng.borrow(), pid, cpos);
    }

    Ok(())
}

pub fn update_view(mut eng: EngineRef, cid: ClientId) {
    let now = eng.now();

    let old_region = unwrap_or!(eng.vision().client_view_area(cid));
    let old_pid = unwrap_or!(eng.vision().client_view_plane(cid));

    let (new_stable_pid, new_region, pawn_id) = {
        // TODO: warn on None? - may indicate inconsistency between World and Vision
        let client = unwrap_or!(eng.world().get_client(cid));

        // TODO: make sure return is the right thing to do on None
        let pawn = unwrap_or!(client.pawn());

        (pawn.stable_plane_id(),
         vision::vision_region(pawn.pos(now)),
         pawn.id())
    };
    let new_pid = chunks::Fragment::get_plane_id(&mut eng.as_chunks_fragment(), new_stable_pid);

    let plane_change = new_pid != old_pid;

    // un/load_chunk use HiddenWorldFragment, so do the calls in this specific order to make sure
    // the chunks being un/loaded are actually not in the client's vision.

    for cpos in new_region.points().filter(|&p| !old_region.contains(p) || plane_change) {
        logic::chunks::load_chunk(eng.borrow(), new_pid, cpos);
    }

    vision::Fragment::set_client_area(&mut eng.as_vision_fragment(), cid, new_pid, new_region);

    for cpos in old_region.points().filter(|&p| !new_region.contains(p) || plane_change) {
        logic::chunks::unload_chunk(eng.borrow(), old_pid, cpos);
    }

    // TODO: +scalar(2) hack
    let old_cpos = old_region.min + scalar(2);
    let new_cpos = new_region.min + scalar(2);
    eng.chat_mut().set_client_location(cid, old_pid, old_cpos, new_pid, new_cpos);

    eng.messages().send_client(cid, ClientResponse::SyncStatus(SyncKind::Ok));

    // TODO: using `with_hooks` here is gross, move schedule_view_update somewhere better
    {
        use world::fragment::Fragment;
        // FIXME: view update
    }
}
