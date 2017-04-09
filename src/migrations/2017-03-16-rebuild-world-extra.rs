/// Rebuild `world.dat` (and some plane data) based on other save data.  This sets the World's
/// stable_id fields based on the highest observed IDs in other files, updates each plane's
/// `saved_chunks` table, and regenerates the ward and teleporter tables in world.extra.
///
/// Usage: ./2017-03-16-rebuild-world input-save output-dir

extern crate server_bundle;
extern crate server_config;
extern crate server_extra;
extern crate server_types;

use server_types::*;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use server_bundle::flat::{Flat, FlatView};
use server_bundle::types::Bundle;
use server_extra::Value;


fn for_each_file<F: FnMut(&Path)>(base: &str, mut f: F) {
    for ent in fs::read_dir(base).unwrap() {
        let ent = ent.unwrap();
        if !ent.file_type().unwrap().is_file() {
            continue;
        }
        let path = ent.path();
        f(&path);
    }
}

fn read_bundle<P: AsRef<Path>+?Sized>(path: &P) -> Bundle {
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();

    let view = FlatView::from_bytes(&buf).unwrap();
    view.unflatten_bundle()
}

fn write_bundle<P: AsRef<Path>+?Sized>(path: &P, b: &Bundle) {
    let mut f = Flat::new();
    f.flatten_bundle(b);
    f.write(&mut File::create(path).unwrap()).unwrap();
}

fn find_template_id(b: &Bundle, name: &str) -> Option<u32> {
    for (i, n) in b.templates.iter().enumerate() {
        if &**n == name {
            return Some(i as u32);
        }
    }
    None
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir_in = &args[1];
    let dir_out = &args[2];

    let mut world_bundle = read_bundle(&format!("{}/world.dat", dir_in));
    let mut plane_bundle = read_bundle(&format!("{}/planes/2.plane", dir_in));


    // Clients

    let mut entity_name_map = HashMap::new();

    for_each_file(&format!("{}/clients", dir_in), |path| {
        let b = read_bundle(path);

        for e in b.entities.iter() {
            if e.stable_id != 0 {
                entity_name_map.insert(Stable::<EntityId>::new(e.stable_id),
                                       b.clients[0].name.to_owned());
            }
        }
    });


    // Terrain chunks

    let mut ward_info = Vec::new();

    let mut teleport_networks = HashMap::new();

    for_each_file(&format!("{}/terrain_chunks", dir_in), |path| {
        let b = read_bundle(path);

        if let Some(ward_id) = find_template_id(&b, "ward") {
            for s in b.structures.iter() {
                if s.template != ward_id {
                    continue;
                }
                println!("found ward at {:?}, extra = {:?}", s.pos, s.extra);

                let owner = match s.extra.get("owner").unwrap().unwrap_value() {
                    Value::StableEntityId(id) => id,
                    _ => panic!("unexpected value for ward owner"),
                };

                let owner_name = match entity_name_map.get(&owner) {
                    Some(name) => (**name).to_owned(),
                    None => {
                        println!("can't find name for ward owner {:?}", owner);
                        format!("[unknown entity {}]", owner.unwrap())
                    },
                };
                ward_info.push((s.pos, owner, owner_name));
            }
        }

        if let Some(teleporter_id) = find_template_id(&b, "teleporter") {
            for s in b.structures.iter() {
                if s.template != teleporter_id {
                    continue;
                }
                println!("found teleporter at {:?}, extra = {:?}", s.pos, s.extra);

                let name = s.extra.get("name").unwrap().unwrap_value().as_str().unwrap();
                let network = s.extra.get("network").unwrap().unwrap_value().as_str().unwrap();
                let pos = s.pos;

                teleport_networks.entry(network).or_insert_with(HashMap::new)
                                 .insert(name, pos);
            }
        }
    });


    {
        let extra = &mut plane_bundle.planes[0].extra;
        let mut h = extra.set_hash("ward_info");

        for (pos, id, name) in ward_info {
            let id_str = format!("{}", id.unwrap());
            let mut h2 = h.borrow().set_hash(&id_str);
            h2.borrow().set("pos", Value::V3(pos));
            h2.borrow().set("name", Value::Str(name));
        }

        let mut h2 = h.borrow().set_hash("server");
        h2.borrow().set("pos", Value::V3(scalar(0)));
        h2.borrow().set("name", Value::Str("the server".to_owned()));
    }

    {
        let w = world_bundle.world.as_mut().unwrap();

        {
            let mut h = w.extra.get_or_set_hash("teleport_networks");
            for (network, map) in teleport_networks {
                let mut h2 = h.borrow().get_or_set_hash(&network);
                for (name, tile_pos) in map {
                    let opt_old_pos = h2.borrow().get(&name).and_then(|v| v.as_value());
                    if let Some(Value::V3(old_pos)) = opt_old_pos {
                        let old_tile_pos = old_pos.div_floor(scalar(32));
                        if (tile_pos - old_tile_pos).abs().max() <= 1 {
                            println!("teleporter {:?} {:?}: already connected", network, name);
                            continue;
                        }
                    }
                    // Otherwise, the teleporter isn't listed yet.
                    let pos = tile_pos * scalar(32) + V3::new(32, 0, 0);
                    println!("teleporter {:?} {:?}: connecting to {:?}",
                             network, name, pos);
                    h2.borrow().set(&name, Value::V3(pos));
                }
            }
        }

        w.extra.set_hash("ward_perms");

        println!("{:?}", w.extra);
    }

    write_bundle(&format!("{}/world.dat", dir_out), &world_bundle);
    write_bundle(&format!("{}/2.plane", dir_out), &plane_bundle);
}
