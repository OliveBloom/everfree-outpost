use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;

pub struct Variant {
    pub name: String,
    pub global_id: u32,
}

pub struct SpritePart {
    pub name: String,
    variants: Vec<Variant>,
    name_to_id: HashMap<String, u32>,
}

pub struct SpritePartData {
    parts: Vec<SpritePart>,
    name_to_id: HashMap<String, u32>,
}

impl SpritePartData {
    pub fn from_json(json: Json) -> Result<SpritePartData, ParseError> {
        let parts = expect!(json.as_array(),
                            "found non-array at top level");

        let mut by_id = Vec::with_capacity(parts.len());
        let mut name_to_id = HashMap::new();

        for (i, part) in parts.iter().enumerate() {
            let part_name = get_convert!(part, "name", as_string,
                                         "for sprite part {}", i);
            let variants = get_convert!(part, "variants", as_array,
                                        "for sprite part {} ({})", i, part_name);

            let mut variant_by_id = Vec::with_capacity(variants.len());
            let mut variant_name_to_id = HashMap::new();

            for (j, variant) in variants.iter().enumerate() {
                if variant.is_null() {
                    variant_by_id.push(Variant {
                        name: "".to_owned(),
                        global_id: -1_i32 as u32,
                    });
                    continue;
                }

                let variant_name = get_convert!(variant, "name", as_string,
                                        "for variant {} of sprite part {} ({})",
                                        j, i, part_name);
                let global_id = get_convert!(variant, "global_id", as_i64,
                                             "for variant {} ({}) of sprite part {} ({})",
                                             j, variant_name, i, part_name);

                variant_by_id.push(Variant {
                    name: variant_name.to_owned(),
                    global_id: global_id as u32,
                });
                variant_name_to_id.insert(variant_name.to_owned(), j as u32);
            }

            by_id.push(SpritePart {
                name: part_name.to_owned(),
                variants: variant_by_id,
                name_to_id: variant_name_to_id,
            });
            name_to_id.insert(part_name.to_owned(), i as u32);
        }

        Ok(SpritePartData {
            parts: by_id,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn part(&self, id: u32) -> &SpritePart {
        self.get_part(id).unwrap()
    }

    pub fn get_part(&self, id: u32) -> Option<&SpritePart> {
        self.parts.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> u32 {
        self.find_id(name).unwrap()
    }

    pub fn find_id(&self, name: &str) -> Option<u32> {
        self.name_to_id.get(name).map(|&x| x)
    }
}

impl SpritePart {
    pub fn len(&self) -> usize {
        self.variants.len()
    }

    pub fn variant(&self, id: u32) -> &Variant {
        self.get_variant(id).unwrap()
    }

    pub fn get_variant(&self, id: u32) -> Option<&Variant> {
        self.variants.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> u32 {
        self.find_id(name).unwrap()
    }

    pub fn find_id(&self, name: &str) -> Option<u32> {
        self.name_to_id.get(name).map(|&x| x)
    }
}
