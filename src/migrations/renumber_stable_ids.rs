/// Assign new (sequential) stable IDs to all objects that currently have them.  This includes
/// updating all references to stable IDs in objects and extras, as well as updating the "next ID"
/// counters in world.dat.
///
/// Usage: ./renumber_stable_ids old_dist new_dist

extern crate server_bundle;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use server_bundle::flat::{self, FlatView, FlatViewMut};

fn for_each_file<F: FnMut(&Path, &str, &str)>(base: &str, mut f: F) {
    //f(Path::new(&format!("{}/world.dat", base)), ".", "world.dat");
    for &dir in &["clients", "planes", "terrain_chunks"] {
        for ent in fs::read_dir(&format!("{}/save/{}", base, dir)).unwrap() {
            let ent = ent.unwrap();
            if !ent.file_type().unwrap().is_file() {
                continue;
            }
            let path = ent.path();
            f(&path,
              path.parent().unwrap().file_name().unwrap().to_str().unwrap(),
              path.file_name().unwrap().to_str().unwrap());
        }
    }
}

fn record(map: &mut HashMap<u64, u64>, stable_id: u64) {
    if stable_id == 0 {
        return;
    }
    if map.contains_key(&stable_id) {
        return;
    }
    let new_id = map.len() as u64 + 1;
    map.insert(stable_id, new_id);
}

fn replace(map: &HashMap<u64, u64>, id: &mut u64) {
    if *id == 0 {
        return;
    }
    let old_id = *id;
    *id = map[&old_id];
}

struct IdMaps {
    clients: HashMap<u64, u64>,
    entities: HashMap<u64, u64>,
    inventories: HashMap<u64, u64>,
    planes: HashMap<u64, u64>,
    terrain_chunks: HashMap<u64, u64>,
    structures: HashMap<u64, u64>,
}

impl IdMaps {
    fn new() -> IdMaps {
        IdMaps {
            clients: HashMap::new(),
            entities: HashMap::new(),
            inventories: HashMap::new(),
            planes: HashMap::new(),
            terrain_chunks: HashMap::new(),
            structures: HashMap::new(),
        }
    }

    fn record(&mut self, view: FlatView) {
        for c in view.clients {
            record(&mut self.clients, c.stable_id);
        }
        for e in view.entities {
            record(&mut self.entities, e.stable_id);
        }
        for i in view.inventories {
            record(&mut self.inventories, i.stable_id);
        }
        for p in view.planes {
            record(&mut self.planes, p.stable_id);
        }
        for tc in view.terrain_chunks {
            record(&mut self.terrain_chunks, tc.stable_id);
        }
        for s in view.structures {
            record(&mut self.structures, s.stable_id);
        }
    }

    fn replace(&self, view: &mut FlatViewMut) {
        if let Some(ref mut w) = view.world {
            w.next_client = self.clients.len() as u64 + 1;
            w.next_entity = self.entities.len() as u64 + 1;
            w.next_inventory = self.inventories.len() as u64 + 1;
            w.next_plane = self.planes.len() as u64 + 1;
            w.next_terrain_chunk = self.terrain_chunks.len() as u64 + 1;
            w.next_structure = self.structures.len() as u64 + 1;
            self.replace_extra(&mut w.extra, view.large_ints);
        }

        for c in &mut *view.clients {
            replace(&self.clients, &mut c.stable_id);
            self.replace_extra(&mut c.extra, view.large_ints);
        }
        for e in &mut *view.entities {
            replace(&self.entities, &mut e.stable_id);
            replace(&self.planes, &mut e.stable_plane);
            self.replace_extra(&mut e.extra, view.large_ints);
        }
        for i in &mut *view.inventories {
            replace(&self.inventories, &mut i.stable_id);
            self.replace_extra(&mut i.extra, view.large_ints);
        }
        for p in &mut *view.planes {
            replace(&self.planes, &mut p.stable_id);
            self.replace_extra(&mut p.extra, view.large_ints);
        }
        for tc in &mut *view.terrain_chunks {
            replace(&self.terrain_chunks, &mut tc.stable_id);
            replace(&self.planes, &mut tc.stable_plane);
            self.replace_extra(&mut tc.extra, view.large_ints);
        }
        for s in &mut *view.structures {
            replace(&self.structures, &mut s.stable_id);
            replace(&self.planes, &mut s.stable_plane);
            self.replace_extra(&mut s.extra, view.large_ints);
        }

        for e in &mut *view.extras {
            self.replace_extra(e, view.large_ints);
        }
        for e in &mut *view.hash_entries {
            self.replace_extra(&mut e.value, view.large_ints);
        }
        for c in &mut *view.loaded_chunks {
            replace(&self.terrain_chunks, &mut c.tcid);
        }
    }

    fn replace_extra(&self, e: &mut flat::FlatExtra, large_ints: &mut [u64]) {
        let tag = flat::Tag::from_primitive(e.tag).unwrap();
        match tag {
            flat::Tag::StableClientId =>
                replace(&self.clients, &mut large_ints[e.data as usize]),
            flat::Tag::StableEntityId =>
                replace(&self.entities, &mut large_ints[e.data as usize]),
            flat::Tag::StableInventoryId =>
                replace(&self.inventories, &mut large_ints[e.data as usize]),
            flat::Tag::StablePlaneId =>
                replace(&self.planes, &mut large_ints[e.data as usize]),
            flat::Tag::StableTerrainChunkId =>
                replace(&self.terrain_chunks, &mut large_ints[e.data as usize]),
            flat::Tag::StableStructureId =>
                replace(&self.structures, &mut large_ints[e.data as usize]),
            _ => {},
        }
    }
}

fn main() {
    let mut maps = IdMaps::new();

    maps.planes.insert(1, 1);
    maps.planes.insert(2, 2);

    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let base_in = &args[1];
    let base_out = &args[2];

    for_each_file(base_in, |path, _, _| {
        println!("[SCAN] {:?}", path);
        let mut buf = Vec::new();
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();
        let view = FlatView::from_bytes(&buf).unwrap();
        maps.record(view);
    });

    for_each_file(base_in, |path, dir, name| {
        let mut buf = Vec::new();
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();

        let new_path = {
            let mut view = FlatViewMut::from_bytes(&mut buf).unwrap();
            maps.replace(&mut view);

            match dir {
                "clients" => format!("{}/save/{}/{}", base_out, dir, name),
                "planes" => format!("{}/save/planes/{:x}.plane",
                                    base_out, view.planes[0].stable_id),
                "terrain_chunks" => format!("{}/save/terrain_chunks/{:x}.terrain_chunk",
                                            base_out, view.terrain_chunks[0].stable_id),
                _ => unreachable!(),
            }
        };
        println!("[OUT] {:?}", new_path);
        File::create(&new_path).unwrap().write_all(&buf).unwrap();
    });
}
