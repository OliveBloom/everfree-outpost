use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;

pub struct Animation {
    pub name: String,
    pub framerate: u32,
    pub length: u32,
}

pub struct AnimationData {
    animations: Vec<Animation>,
    name_to_id: HashMap<String, AnimId>,
}

impl AnimationData {
    pub fn from_json(json: Json) -> Result<AnimationData, ParseError> {
        let animations = expect!(json.as_array(),
                                "found non-array at top level");

        let mut by_id = Vec::with_capacity(animations.len());
        let mut name_to_id = HashMap::new();

        for (i, animation) in animations.iter().enumerate() {
            let name = get_convert!(animation, "name", as_string,
                                    "for animation {}", i);
            let framerate = get_convert!(animation, "framerate", as_i64,
                                        "for animation {} ({})", i, name);
            let length = get_convert!(animation, "length", as_i64,
                                      "for animation {} ({})", i, name);

            by_id.push(Animation {
                name: name.to_owned(),
                framerate: framerate as u32,
                length: length as u32,
            });
            name_to_id.insert(name.to_owned(), i as AnimId);
        }

        Ok(AnimationData {
            animations: by_id,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        self.animations.len()
    }

    pub fn animation(&self, id: AnimId) -> &Animation {
        self.get_animation(id).unwrap()
    }

    pub fn get_animation(&self, id: AnimId) -> Option<&Animation> {
        self.animations.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> AnimId {
        self.find_id(name).unwrap_or_else(|| panic!("unknown animation id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<AnimId> {
        self.name_to_id.get(name).map(|&x| x)
    }

    pub fn get_by_id(&self, name: &str) -> &Animation {
        self.animation(self.get_id(name))
    }
}
