use std::str::FromStr;

use physics::CHUNK_SIZE;
use server_bundle::builder::Builder;
use server_bundle::types::Bundle;
use server_config::Data;
use server_extra::Value;
use server_types::*;
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
                    "loot" => {
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
                    },

                    _ => panic!("unrecognized GenStructure extras: {:?}", k),
                }
            }
        }
    }

    b.finish()
}
