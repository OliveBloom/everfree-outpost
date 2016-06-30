use std::str::FromStr;

use physics::CHUNK_SIZE;
use server_bundle::builder::{Builder, StructureBuilder};
use server_bundle::types::Bundle;
use server_config::Data;
use server_extra::Value;
use server_types::*;
use server_world_types::flags::S_HAS_IMPORT_HOOK;
use terrain_gen::GenChunk;




pub fn gen_chunk_to_bundle(data: &Data,
                           gc: GenChunk,
                           pid: Stable<PlaneId>,
                           cpos: V2) -> Bundle {
    let mut b = Builder::new(data);

    let mut blocks = Box::new(EMPTY_CHUNK);
    for i in 0 .. blocks.len() {
        blocks[i] = b.block_id(gc.blocks[i]);
    }

    {
        let mut tc = b.terrain_chunk();
        tc.stable_plane(pid)
          .cpos(cpos)
          .blocks(blocks);

        for gs in gc.structures {
            let mut s = tc.structure_();
            s.stable_plane(pid)
             .pos(cpos.extend(0) * scalar(CHUNK_SIZE) + gs.pos)
             .template_id(gs.template);

            for (k, v) in &gs.extra {
                match k as &str {
                    "loot" => apply_loot(&mut s, v),
                    "gem_puzzle_slot" => apply_gem_puzzle_slot(&mut s, v),
                    "gem_puzzle_door" => apply_gem_puzzle_door(&mut s, v),

                    _ => panic!("unrecognized GenStructure extras: {:?}", k),
                }
            }
        }
    }

    b.finish()
}

fn apply_loot(s: &mut StructureBuilder, v: &str) {
    let iid = {
        let mut i = s.inventory_();
        i.size(30);
        for (slot, part) in v.split(',').enumerate() {
            if part == "" {
                continue;
            }

            let idx = part.find(':').unwrap();
            let (name, colon_count) = part.split_at(idx);
            let count: u8 = FromStr::from_str(&colon_count[1..]).unwrap();
            i.item(slot as u8, name, count);
        }
        i.id()
    };
    s.extra_()
     .get_or_set_hash("inv")
     .set("main", Value::InventoryId(iid));
}

fn apply_gem_puzzle_slot(s: &mut StructureBuilder, v: &str) {
    let mut iter = v.split(',');
    let puzzle_id = iter.next().unwrap();
    let slot = FromStr::from_str(iter.next().unwrap()).unwrap();
    let init = iter.next().unwrap();

    s.flags(S_HAS_IMPORT_HOOK);
    let mut e = s.extra_();
    e.set("puzzle_id", Value::Str(puzzle_id.to_owned()));
    e.set("puzzle_slot", Value::Int(slot));
    e.set("puzzle_init", Value::Str(init.to_owned()));
}

fn apply_gem_puzzle_door(s: &mut StructureBuilder, v: &str) {
    let mut iter = v.split(',');
    let puzzle_id = iter.next().unwrap();

    s.flags(S_HAS_IMPORT_HOOK);
    let mut e = s.extra_();
    e.set("puzzle_id", Value::Str(puzzle_id.to_owned()));
    e.set("puzzle_init", Value::Bool(true));
}
