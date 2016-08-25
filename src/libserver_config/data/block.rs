use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;


pub struct BlockDef {
    pub name: String,
    pub flags: BlockFlags,
}

pub struct BlockData {
    blocks: Vec<BlockDef>,
    name_to_id: HashMap<String, BlockId>,
}

impl BlockData {
    pub fn from_json(json: Json) -> Result<BlockData, ParseError> {
        let blocks = expect!(json.as_array(),
                                "found non-array at top level");

        let mut block_defs = Vec::with_capacity(blocks.len());
        let mut name_to_id = HashMap::new();

        for (i, block) in blocks.iter().enumerate() {
            let name = get_convert!(block, "name", as_string,
                                    "for block {}", i);
            let raw_flags = get_convert!(block, "flags", as_i64,
                                         "for block {} ({})", i, name);

            let flags = match BlockFlags::from_bits(raw_flags as _) {
                Some(x) => x,
                None => return fail!("invalid flags 0x{:x} for block {} ({})",
                                     raw_flags, i, name),
            };

            block_defs.push(BlockDef {
                name: name.to_owned(),
                flags: flags,
            });
            name_to_id.insert(name.to_owned(), i as BlockId);
        }

        Ok(BlockData {
            blocks: block_defs,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        return self.blocks.len()
    }

    pub fn block(&self, id: BlockId) -> &BlockDef {
        self.get_block(id)
            .unwrap_or_else(|| panic!("invalid block ID: {}", id))
    }

    pub fn get_block(&self, id: BlockId) -> Option<&BlockDef> {
        self.blocks.get(id as usize)
    }

    pub fn shape(&self, id: BlockId) -> Shape {
        self.blocks.get(id as usize).map_or(Shape::Empty, |b| b.flags.shape())
    }

    pub fn name(&self, id: BlockId) -> &str {
        &self.blocks[id as usize].name
    }

    pub fn get_name(&self, id: BlockId) -> Option<&str> {
        self.blocks.get(id as usize).map(|b| &b.name as &str)
    }

    pub fn get_id(&self, name: &str) -> BlockId {
        self.find_id(name).unwrap_or_else(|| panic!("unknown block id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<BlockId> {
        self.name_to_id.get(name).map(|&x| x)
    }
}
