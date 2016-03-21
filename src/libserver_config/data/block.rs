use std::collections::HashMap;
use std::iter::repeat;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;

pub struct BlockData {
    shapes: Vec<Shape>,
    names: Vec<String>,
    name_to_id: HashMap<String, BlockId>,
}

impl BlockData {
    pub fn from_json(json: Json) -> Result<BlockData, ParseError> {
        let blocks = expect!(json.as_array(),
                                "found non-array at top level");

        let mut shapes = repeat(Shape::Empty).take(blocks.len()).collect::<Vec<_>>();
        let mut names = Vec::with_capacity(shapes.len());
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
            shapes[i] = shape;
            names.push(name.to_owned());
            name_to_id.insert(name.to_owned(), i as BlockId);
        }

        Ok(BlockData {
            shapes: shapes,
            names: names,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        return self.names.len()
    }

    pub fn shape(&self, id: BlockId) -> Shape {
        self.shapes.get(id as usize).map(|&x| x).unwrap_or(Shape::Empty)
    }

    pub fn name(&self, id: BlockId) -> &str {
        &*self.names[id as usize]
    }

    pub fn get_name(&self, id: BlockId) -> Option<&str> {
        self.names.get(id as usize).map(|s| &**s)
    }

    pub fn get_id(&self, name: &str) -> BlockId {
        self.find_id(name).unwrap_or_else(|| panic!("unknown block id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<BlockId> {
        self.name_to_id.get(name).map(|&x| x)
    }
}
