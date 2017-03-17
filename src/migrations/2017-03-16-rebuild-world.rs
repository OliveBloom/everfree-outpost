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

fn read_plane_bundles(dir_in: &str) -> HashMap<Stable<PlaneId>, Bundle> {
    let mut h = HashMap::new();

    for_each_file(&format!("{}/planes", dir_in), |path| {
        let b = read_bundle(path);
        let stable = b.planes[0].stable_id;
        h.insert(Stable::new(stable), b);
    });

    h
}

#[derive(Debug)]
struct Counters {
    client: u64,
    entity: u64,
    inventory: u64,
    plane: u64,
    terrain_chunk: u64,
    structure: u64,
}

impl Counters {
    pub fn update(&mut self, b: &Bundle) {
        for c in b.clients.iter() {
            self.client = cmp::max(self.client, c.stable_id);
        }
        for e in b.entities.iter() {
            self.entity = cmp::max(self.entity, e.stable_id);
        }
        for i in b.inventories.iter() {
            self.inventory = cmp::max(self.inventory, i.stable_id);
        }
        for p in b.planes.iter() {
            self.plane = cmp::max(self.plane, p.stable_id);
        }
        for tc in b.terrain_chunks.iter() {
            self.terrain_chunk = cmp::max(self.terrain_chunk, tc.stable_id);
        }
        for i in b.inventories.iter() {
            self.inventory = cmp::max(self.inventory, i.stable_id);
        }
    }
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
    let mut plane_bundles = read_plane_bundles(dir_in);

    println!("loaded world and {} planes", plane_bundles.len());

    let mut counters = Counters {
        client: 0,
        entity: 0,
        inventory: 0,
        plane: 0,
        terrain_chunk: 0,
        structure: 0,
    };


    // Clients

    let mut entity_name_map = HashMap::new();

    for_each_file(&format!("{}/clients", dir_in), |path| {
        let b = read_bundle(path);
        counters.update(&b);

        for e in b.entities.iter() {
            if e.stable_id != 0 {
                entity_name_map.insert(Stable::<EntityId>::new(e.stable_id),
                                       b.clients[0].name.to_owned());
            }
        }
    });


    // Planes

    for (_, b) in plane_bundles.iter() {
        assert!(b.planes.len() == 1);
        counters.update(b);
    }


    // Terrain chunks

    let mut saved_chunk_lists = HashMap::new();
    for (&k, _) in &plane_bundles {
        saved_chunk_lists.insert(k, Vec::new());
    }

    let mut tc_ids = HashSet::new();

    let mut ward_info = Vec::new();

    let mut teleport_networks = HashMap::new();

    for_each_file(&format!("{}/terrain_chunks", dir_in), |path| {
        let b = read_bundle(path);
        counters.update(&b);
        assert!(b.terrain_chunks.len() == 1);

        let tc = &b.terrain_chunks[0];
        saved_chunk_lists.get_mut(&tc.stable_plane).unwrap()
                         .push((tc.cpos, Stable::new(tc.stable_id)));

        assert!(!tc_ids.contains(&tc.stable_id));
        tc_ids.insert(tc.stable_id);

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

                ward_info.push((s.pos, owner, &entity_name_map[&owner]));
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
                let pos = s.pos * scalar(32) + V3::new(32, 0, 0);

                teleport_networks.entry(network).or_insert_with(HashMap::new)
                                 .insert(name, pos);
            }
        }
    });

    for (&k, b) in plane_bundles.iter_mut() {
        let saved = saved_chunk_lists.remove(&k).unwrap().into_boxed_slice();
        b.planes[0].saved_chunks = saved;
        println!("plane {:?}: {} saved chunks", k, b.planes[0].saved_chunks.len());
    }


    {
        let extra = &mut plane_bundles.get_mut(&Stable::new(2)).unwrap().planes[0].extra;
        let mut h = extra.set_hash("ward_info");

        for (pos, id, name) in ward_info {
            let id_str = format!("{}", id.unwrap());
            let mut h2 = h.borrow().set_hash(&id_str);
            h2.borrow().set("pos", Value::V3(pos));
            h2.borrow().set("name", Value::Str(name.to_string()));
        }

        let mut h2 = h.borrow().set_hash("server");
        h2.borrow().set("pos", Value::V3(scalar(0)));
        h2.borrow().set("name", Value::Str("the server".to_owned()));
    }


    println!("{:?}", counters);
    println!("{} chunks", tc_ids.len());
    println!("{:?}", plane_bundles[&Stable::new(2)].planes[0].extra);

    {
        let w = world_bundle.world.as_mut().unwrap();
        w.next_client = counters.client + 1;
        w.next_entity = counters.entity + 1;
        w.next_inventory = counters.inventory + 1;
        w.next_plane = counters.plane + 1;
        w.next_terrain_chunk = counters.terrain_chunk + 1;
        w.next_structure = counters.structure + 1;

        {
            let mut h = w.extra.set_hash("teleport_networks");
            for (network, map) in teleport_networks {
                let mut h2 = h.borrow().set_hash(&network);
                for (name, pos) in map {
                    h2.borrow().set(&name, Value::V3(pos));
                }
            }
        }

        w.extra.set_hash("ward_perms");

        println!("{:?}", w.extra);
    }

    write_bundle(&format!("{}/world.dat", dir_out), &world_bundle);
    for (&k, b) in &plane_bundles {
        write_bundle(&format!("{}/{}.plane", dir_out, k.unwrap()), b);
    }
}
