#![crate_name = "generate_terrain"]

extern crate env_logger;
#[macro_use] extern crate log;
extern crate rand;
extern crate rustc_serialize;

extern crate physics;
extern crate server_bundle;
extern crate server_config;
extern crate server_extra;
extern crate server_types;
extern crate server_util;
extern crate server_world_types;
extern crate terrain_gen;

use server_bundle::types::Bundle;
use server_bundle::flat::Flat;
use server_config::{Data, Storage};
use server_types::*;
use terrain_gen::forest::Provider as ForestProvider;
use terrain_gen::dungeon::Provider as DungeonProvider;


use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::u32;
use rand::{XorShiftRng, Rng};
use rustc_serialize::json;

use server_util::bytes::{ReadBytes, WriteBytes};


mod adapter;


struct Context<'d> {
    data: &'d Data,
    forest: ForestProvider<'d>,
    dungeon: DungeonProvider<'d>,
}

impl<'d> Context<'d> {
    fn generate(&mut self, pid: Stable<PlaneId>, cpos: V2) -> Bundle {
        //let start = now();

        let gc =
            if pid == STABLE_PLANE_FOREST {
                self.forest.generate(pid, cpos)
            } else {
                self.dungeon.generate(pid, cpos)
            };

        let b = adapter::gen_chunk_to_bundle(self.data, gc, pid, cpos);

        //let end = now();
        //info!("generated {} {:?} in {} ms", pid.unwrap(), cpos, end - start);

        b
    }
}

const OP_INIT_PLANE: u32 =      0;
const OP_FORGET_PLANE: u32 =    1;
const OP_GEN_PLANE: u32 =       2;
const OP_GEN_CHUNK: u32 =       3;
const OP_SHUTDOWN: u32 =        4;

fn io_main(ctx: &mut Context) -> io::Result<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        let opcode: u32 = try!(stdin.read_bytes());
        match opcode {
            OP_INIT_PLANE => unimplemented!(),
            OP_FORGET_PLANE => unimplemented!(),
            OP_GEN_PLANE => unimplemented!(),
            OP_GEN_CHUNK => {
                let (pid, cpos) = try!(stdin.read_bytes());
                let b = ctx.generate(pid, cpos);

                let mut f = Flat::new();
                f.flatten_bundle(&b);
                let bytes = f.to_bytes();
                let len = bytes.len();
                assert!(len <= u32::MAX as usize);

                try!(stdout.write_bytes(len as u32));
                try!(stdout.write_all(&bytes));
            },
            OP_SHUTDOWN => {
                info!("clean shutdown");
                break;
            },
            _ => panic!("unrecognized opcode: {}", opcode),
        }
    }

    Ok(())
}


fn read_json(mut file: File) -> json::Json {
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    json::Json::from_str(&content).unwrap()
}

fn load_config(path: &Path) -> (Data, Storage) {
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

    (data, storage)
}

fn main() {
    env_logger::init().unwrap();


    let args = env::args().collect::<Vec<_>>();
    let (data, storage) = load_config(Path::new(&args[1]));


    let mut rng: XorShiftRng = rand::random();
    let mut ctx = Context {
        data: &data,
        forest: ForestProvider::new(&data, &storage, rng.gen()),
        dungeon: DungeonProvider::new(&data, &storage, rng.gen()),
    };

    match io_main(&mut ctx) {
        Ok(()) => {},
        Err(e) => error!("io error: {:?}", e),
    }
}
