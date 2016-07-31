use std::collections::HashMap;
use rustc_serialize::json::Json;

use super::ParseError;

pub struct SpriteLayerData {
    names: Vec<String>,
    name_to_id: HashMap<String, u32>,
}

impl SpriteLayerData {
    pub fn from_json(json: Json) -> Result<SpriteLayerData, ParseError> {
        let layers = expect!(json.as_array(),
                             "found non-array at top level");

        let mut names = Vec::with_capacity(layers.len());
        let mut name_to_id = HashMap::new();

        for (i, layer) in layers.iter().enumerate() {
            let name = get_convert!(layer, "name", as_string,
                                    "for sprite layer {}", i);

            names.push(name.to_owned());
            name_to_id.insert(name.to_owned(), i as u32);
        }

        Ok(SpriteLayerData {
            names: names,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn name(&self, id: u32) -> &str {
        &*self.names[id as usize]
    }

    pub fn get_name(&self, id: u32) -> Option<&str> {
        self.names.get(id as usize).map(|s| &**s)
    }

    pub fn get_id(&self, name: &str) -> u32 {
        self.find_id(name).unwrap_or_else(|| panic!("unknown sprite layer id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<u32> {
        self.name_to_id.get(name).map(|&x| x)
    }
}
