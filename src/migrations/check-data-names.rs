/// Check all references to items, templates, etc. in the save file to ensure that those
/// definitions exist in the actual game data.
///
/// Usage: ./check-data-names save dist

extern crate server_bundle;
extern crate server_config;
extern crate rustc_serialize;

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use rustc_serialize::json;
use server_bundle::flat::FlatView;
use server_config::{Storage, Data};

fn for_each_file<F: FnMut(&Path, &str, &str)>(base: &str, mut f: F) {
    f(Path::new(&format!("{}/world.dat", base)), ".", "world.dat");
    for &dir in &["clients", "planes", "terrain_chunks"] {
        for ent in fs::read_dir(&format!("{}/{}", base, dir)).unwrap() {
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


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

fn load_config(path: &Path) -> Data {
    let storage = Storage::new(&path.to_owned());

    let block_json = read_json(storage.open_block_data());
    let item_json = read_json(storage.open_item_data());
    let recipe_json = read_json(storage.open_recipe_data());
    let template_json = read_json(storage.open_template_data());
    let animation_json = read_json(storage.open_animation_data());
    let sprite_layer_json = read_json(storage.open_sprite_layer_data());
    let loot_table_json = read_json(storage.open_loot_table_data());
    let data = Data::from_json(block_json,
                               item_json,
                               recipe_json,
                               template_json,
                               animation_json,
                               sprite_layer_json,
                               loot_table_json).unwrap();

    data
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    println!("{:?}", args);
    let save_path = &args[1];
    let data_path = &args[2];

    let data = load_config(Path::new(data_path));

    for_each_file(save_path, |path, _, _| {
        println!("[SCAN] {:?}", path);
        let mut buf = Vec::new();
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();
        let view = FlatView::from_bytes(&buf).unwrap();
        let bundle = view.unflatten_bundle();

        for name in bundle.anims.iter() {
            if data.animations.find_id(name).is_none() {
                println!("missing animation: {}", name);
            }
        }

        for name in bundle.items.iter() {
            if data.item_data.find_id(name).is_none() {
                println!("missing item: {}", name);
            }
        }

        for name in bundle.blocks.iter() {
            if data.block_data.find_id(name).is_none() {
                println!("missing block: {}", name);
            }
        }

        for name in bundle.templates.iter() {
            if data.structure_templates.find_id(name).is_none() {
                println!("missing template: {}", name);
            }
        }
    });
}
