extern crate server_bundle;
extern crate server_extra;

use std::env;
use std::fs::File;
use std::io::Read;

use server_extra as extra;
use server_extra::Extra;
use server_bundle::flat;
use server_bundle::types::Bundle;

fn load_bundle(path: &str) -> Bundle {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let flat = flat::FlatView::from_bytes(&buf).unwrap();
    flat.unflatten_bundle()
}

fn dump_extra(e: &Extra) {
    if e.len() == 0 {
        println!("    (empty)");
        return;
    }
    for (k, v) in e.iter() {
        print!("    {:?}: ", k);
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, 2),
            extra::View::Hash(h) => dump_extra_hash(h, 2),
        }
    }
}

fn dump_extra_value(v: extra::Value) {
    use server_extra::Value;
    match v {
        Value::Null => println!("!!null _"),
        Value::Bool(b) => println!("!!bool {}", b),
        Value::Int(i) => println!("{}", i),
        Value::Float(f) => println!("{}", f),
        Value::Str(s) => println!("{:?}", s),

        Value::ClientId(id) => println!("!!client {}", id.unwrap()),
        Value::EntityId(id) => println!("!!entity {}", id.unwrap()),
        Value::InventoryId(id) => println!("!!inventory {}", id.unwrap()),
        Value::PlaneId(id) => println!("!!plane {}", id.unwrap()),
        Value::TerrainChunkId(id) => println!("!!terrain_chunk {}", id.unwrap()),
        Value::StructureId(id) => println!("!!structure {}", id.unwrap()),

        Value::StableClientId(id) => println!("!!stable_client {}", id.unwrap()),
        Value::StableEntityId(id) => println!("!!stable_entity {}", id.unwrap()),
        Value::StableInventoryId(id) => println!("!!stable_inventory {}", id.unwrap()),
        Value::StablePlaneId(id) => println!("!!stable_plane {}", id.unwrap()),
        Value::StableTerrainChunkId(id) => println!("!!stable_terrain_chunk {}", id.unwrap()),
        Value::StableStructureId(id) => println!("!!stable_structure {}", id.unwrap()),

        Value::V2(v) => println!("!!v2 [{}, {}]", v.x, v.y),
        Value::V3(v) => println!("!!v3 [{}, {}, {}]", v.x, v.y, v.z),
        Value::Region2(r) => println!("!!region2 [[{}, {}], [{}, {}]]",
                                      r.min.x, r.min.y,
                                      r.max.x, r.max.y),
        Value::Region3(r) => println!("!!region3 [[{}, {}, {}], [{}, {}, {}]]",
                                      r.min.x, r.min.y, r.min.z,
                                      r.max.x, r.max.y, r.max.z),
    }
}

fn dump_extra_array(a: extra::ArrayView, indent: usize) {
    println!("");
    for v in a.iter() {
        for _ in 0 .. indent {
            print!("    ");
        }
        print!("- ");
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, indent + 1),
            extra::View::Hash(h) => dump_extra_hash(h, indent + 1),
        }
    }
}

fn dump_extra_hash(h: extra::HashView, indent: usize) {
    println!("");
    if h.len() == 0 {
        for _ in 0 .. indent {
            print!("    ");
        }
        println!("(empty)");
        return;
    }
    for (k, v) in h.iter() {
        for _ in 0 .. indent {
            print!("    ");
        }
        print!("{:?}: ", k);
        match v {
            extra::View::Value(v) => dump_extra_value(v),
            extra::View::Array(a) => dump_extra_array(a, indent + 1),
            extra::View::Hash(h) => dump_extra_hash(h, indent + 1),
        }
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let path = &args[1];

    let bundle = load_bundle(path);

    if let Some(ref w) = bundle.world {
        println!("World extra:");
        dump_extra(&w.extra);
    }

    for (i, c) in bundle.clients.iter().enumerate() {
        println!("Client {} ({:x}) extra:", i, c.stable_id);
        dump_extra(&c.extra);
    }

    for (i, e) in bundle.entities.iter().enumerate() {
        println!("Entity {} ({:x}) extra:", i, e.stable_id);
        dump_extra(&e.extra);
    }

    for (i, inv) in bundle.inventories.iter().enumerate() {
        println!("Inventory {} ({:x}) extra:", i, inv.stable_id);
        dump_extra(&inv.extra);
    }

    for (i, p) in bundle.planes.iter().enumerate() {
        println!("Plane {} ({:x}) extra:", i, p.stable_id);
        dump_extra(&p.extra);
    }

    for (i, tc) in bundle.terrain_chunks.iter().enumerate() {
        println!("Terrain chunk {} ({:x}) extra:", i, tc.stable_id);
        dump_extra(&tc.extra);
    }

    for (i, s) in bundle.structures.iter().enumerate() {
        println!("Structure {} ({:x}) extra:", i, s.stable_id);
        dump_extra(&s.extra);
    }
}
