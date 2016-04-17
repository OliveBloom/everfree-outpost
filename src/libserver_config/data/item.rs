use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;

pub struct ItemData {
    names: Vec<String>,
    name_to_id: HashMap<String, ItemId>,
}

impl ItemData {
    pub fn from_json(json: Json) -> Result<ItemData, ParseError> {
        let items = expect!(json.as_array(),
                            "found non-array at top level");

        let mut names = Vec::with_capacity(items.len());
        let mut name_to_id = HashMap::new();

        for (i, item) in items.iter().enumerate() {
            let name = get_convert!(item, "name", as_string,
                                    "for item {}", i);

            names.push(name.to_owned());
            name_to_id.insert(name.to_owned(), i as ItemId);
        }

        Ok(ItemData {
            names: names,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn name(&self, id: ItemId) -> &str {
        &*self.names[id as usize]
    }

    pub fn get_name(&self, id: ItemId) -> Option<&str> {
        self.names.get(id as usize).map(|s| &**s)
    }

    pub fn get_id(&self, name: &str) -> ItemId {
        self.find_id(name).unwrap_or_else(|| panic!("unknown item id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<ItemId> {
        self.name_to_id.get(name).map(|&x| x)
    }
}
