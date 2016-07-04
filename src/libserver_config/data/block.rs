use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;


bitflags! {
    pub flags BlockFlags: u16 {
        // Cell parts. In a given cell, there can be at most one block for each part.
        //
        // There are two types of floor, called "floor" and "subfloor".  Terrain uses only
        // "subfloor", and structures use only "floor".  This allows player-built floors to overlap
        // the terrain.
        /// The block includes a subfloor (identical to a normal floor, but doesn't conflict).
        const B_SUBFLOOR =      0x0001,
        /// The block includdes a floor.
        const B_FLOOR =         0x0002,
        /// The block includes a solid component.
        const B_SOLID =         0x0004,
        const B_PART_MASK =     0x0007,

        /// If `B_SOLID` is set, these bits determine the actual shape for physics purposes.
        /// Otherwise, it's determined by the presence of `B_SUBFLOOR` and `B_FLOOR`.
        const B_SHAPE_MASK =    0xf000,

        /// The block stops vision.
        const B_OPAQUE =        0x0008,
        /// The block applies a movespeed adjustment to characters standing in it.  The adjustment
        /// amount is stored separately.
        const B_SPEED_ADJUST =  0x0010,
        /// Makes the block "count" for purposes of player interaction hit-tests, and for
        /// determining whether two structures overlap in the same layer.
        const B_OCCUPIED =      0x0020,
        /// The block can have addons attached to it.
        const B_ADDON_TARGET =  0x0040,
        /// The block is (part of) an addon, which can be attached to `B_ADDON_TARGET`s.
        const B_IS_ADDON =      0x0080,

        //const B_HAS_SCRIPT =    0x0100,   // NYI
    }
}

impl BlockFlags {
    pub fn from_shape(s: Shape) -> BlockFlags {
        match s {
            Shape::Empty => BlockFlags::empty(),
            Shape::Floor => B_FLOOR,
            _ => B_SOLID | BlockFlags::from_bits_truncate((s as u8 as u16) << 12),
        }
    }

    pub fn shape(&self) -> Shape {
        if self.contains(B_SOLID) {
            let shape_num = ((*self & B_SHAPE_MASK).bits() >> 12) as usize;
            Shape::from_primitive(shape_num)
                .expect("invalid shape value in B_SHAPE_MASK")
        } else if self.contains(B_FLOOR) || self.contains(B_SUBFLOOR) {
            Shape::Floor
        } else {
            Shape::Empty
        }
    }
}


pub struct BlockDef {
    pub name: String,
    pub flags: BlockFlags,
    /// Speed adjustment percentage.  100 = normal speed.
    pub speed_adjust: u8,
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
            let shape_str = get_convert!(block, "shape", as_string,
                                         "for block {} ({})", i, name);

            let shape = match shape_str {
                "empty" => Shape::Empty,
                "floor" => Shape::Floor,
                "solid" => Shape::Solid,
                "ramp_n" => Shape::RampN,
                _ => return fail!("invalid shape \"{}\" for block {} ({})",
                                  shape_str, i, name),
            };

            block_defs.push(BlockDef {
                name: name.to_owned(),
                flags: BlockFlags::from_shape(shape),
                speed_adjust: 100,
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
