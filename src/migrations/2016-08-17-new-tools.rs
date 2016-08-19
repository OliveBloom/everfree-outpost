/// Replace old tools and structures with new ones in client inventories.  Specifically:
///
///  * `axe` -> `axe/stone`
///  * `pick` -> `pick/stone` (for stacks of <5), or `pick/copper` for larger stacks
///  * `anvil` -> `workbench`
///
/// Usage: ./2016-08-17-new-tools old_clients new_clients

extern crate server_bundle;

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use server_bundle::flat::{Flat, FlatView};


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

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let dir_in = &args[1];
    let dir_out = &args[2];

    for_each_file(dir_in, |path| {
        println!("\nprocessing {:?}", path);
        let path_out = Path::new(dir_out).join(path.file_name().unwrap());

        let mut buf = Vec::new();
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();

        let view = FlatView::from_bytes(&buf).unwrap();
        let mut b = view.unflatten_bundle();

        let mut items = b.items.to_vec();

        let mut pick_id = None;
        for (id, name) in items.iter_mut().enumerate() {
            if &**name == "axe" {
                println!("renamed axe @ {}", id);
                *name = "axe/stone".to_owned().into_boxed_str();
            }
            if &**name == "pick" {
                println!("renamed pick @ {}", id);
                *name = "axe/stone".to_owned().into_boxed_str();
                assert!(pick_id.is_none());
                pick_id = Some(id as u16);
            }
            if &**name == "anvil" {
                println!("renamed anvil @ {}", id);
                *name = "workbench".to_owned().into_boxed_str();
            }
        }

        let mut copper_pick_id = None;
        if let Some(pick_id) = pick_id {
            for inv in b.inventories.iter_mut() {
                for slot in inv.contents.iter_mut() {
                    if slot.id != pick_id || slot.count < 5 {
                        continue;
                    }
                    println!("found pick stack: count = {}", slot.count);

                    if copper_pick_id.is_none() {
                        copper_pick_id = Some(items.len() as u16);
                        items.push("pick/copper".to_owned().into_boxed_str());
                        println!("allocated copper pick @ {}", copper_pick_id.unwrap());
                    }

                    slot.id = copper_pick_id.unwrap();
                    slot.count = 1;
                }
            }
        }

        b.items = items.into_boxed_slice();

        let mut flat = Flat::new();
        flat.flatten_bundle(&b);
        flat.write(&mut File::create(&path_out).unwrap()).unwrap();
        println!("wrote output to {:?}", path_out);
    });
}
