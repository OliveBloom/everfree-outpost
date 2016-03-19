use types::*;

use chunks;
use engine::split::EngineRef;
use logic;
use messages::{ClientResponse, SyncKind};
use world;
use world::Motion;
use world::bundle::{self, Builder};
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
        eng.borrow().unwrap().kick_client(old_cid, "logged in from another location");
    }

    // Load the client bundle
    let mut file = unwrap!(eng.storage().open_client_file(name),
                           "client file not found");
    let b = try!(bundle::read_bundle(&mut file));

    if b.clients.len() != 1 {
        fail!("expected exactly one client in bundle");
    }
    let c = &b.clients[0];
    let b_eid = unwrap!(c.pawn, "client with no pawn is not yet supported");
    let e = unwrap!(b.entities.get(b_eid.unwrap() as usize));

    // TODO: make sure login cannot fail past this point

    // Load the plane and nearby chunks
    // TODO: stop entity motions on logout
    let region = vision::vision_region(e.motion.end_pos);
    let pid = chunks::Fragment::get_plane_id(&mut eng.as_chunks_fragment(), e.stable_plane);
    for cpos in region.points() {
        logic::chunks::load_chunk(eng.borrow(), pid, cpos);
    }

    // Import the client and associated objects into the world
    let importer = bundle::import_bundle(&mut eng.as_world_fragment(), &b);
    let cid = importer.import(&ClientId(0));
    let eid = importer.import(&b_eid);

    // Set up the client to receive messages
    info!("{:?}: logged in as {} ({:?})", wire_id, name, cid);
    eng.messages_mut().add_client(cid, wire_id, name);

    // Send the client's startup messages.
    let cycle_base = (now % DAY_NIGHT_CYCLE_MS as Time) as u32;
    eng.messages_mut().send_client(cid, ClientResponse::Init(Some(eid),
                                                             now,
                                                             cycle_base,
                                                             DAY_NIGHT_CYCLE_MS));

    vision::Fragment::add_client(&mut eng.as_vision_fragment(), cid, pid, region);
    warn_on_err!(eng.script_hooks().call_client_login(eng.borrow(), cid));
    eng.messages().send_client(cid, ClientResponse::SyncStatus(SyncKind::Ok));

    Ok(())
}

pub fn logout(mut eng: EngineRef, cid: ClientId) -> bundle::Result<()> {
    eng.messages_mut().remove_client(cid);

    let old_region = eng.vision().client_view_area(cid);
    let old_pid = eng.vision().client_view_plane(cid);
    vision::Fragment::remove_client(&mut eng.as_vision_fragment(), cid);
    if let (Some(old_region), Some(old_pid)) = (old_region, old_pid) {
        for cpos in old_region.points() {
            logic::chunks::unload_chunk(eng.borrow(), old_pid, cpos);
        }
    }

    // Actually save and destroy the client
    {
        let c = eng.world().client(cid);

        let mut exporter = bundle::Exporter::new(eng.data());
        exporter.add_client(&c);
        let b = exporter.finish();

        let mut file = eng.storage().create_client_file(c.name());
        try!(bundle::write_bundle(&mut file, &b));
    }
    try!(world::Fragment::destroy_client(&mut eng.as_world_fragment(), cid));
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

    eng.messages().send_client(cid, ClientResponse::SyncStatus(SyncKind::Ok));

    // TODO: using `with_hooks` here is gross, move schedule_view_update somewhere better
    {
        use world::fragment::Fragment;
        eng.as_world_fragment().with_hooks(|h| h.schedule_view_update(pawn_id));
    }
}
