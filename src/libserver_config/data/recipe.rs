use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;

pub struct Recipe {
    pub name: String,
    pub inputs: HashMap<ItemId, u8>,
    pub outputs: HashMap<ItemId, u8>,
    pub station: Option<TemplateId>,
}

pub struct RecipeData {
    recipes: Vec<Recipe>,
    name_to_id: HashMap<String, RecipeId>,
}

impl RecipeData {
    pub fn from_json(json: Json) -> Result<RecipeData, ParseError> {
        let recipes_json = expect!(json.as_array(),
                                "found non-array at top level");

        let mut recipes = Vec::with_capacity(recipes_json.len());
        let mut name_to_id = HashMap::new();

        for (i, recipe) in recipes_json.iter().enumerate() {
            let name = get_convert!(recipe, "name", as_string,
                                    "for recipe {}", i);
            let station = match find_convert!(recipe, "station", as_i64,
                                              "for recipe {}", i) {
                Ok(station) => Some(station as TemplateId),
                Err(_) => None,
            };

            fn build_map(list: &[Json], what: &str, i: usize) -> Result<HashMap<ItemId, u8>, ParseError> {
                let mut map = HashMap::new();
                for (j, entry) in list.iter().enumerate() {
                    let entry = expect!(entry.as_array(),
                                        "failed to convert recipe {} {} {}", i, what, j);
                    if entry.len() != 2 {
                        return fail!("bad length for recipe {} {} {}", i, what, j);
                    }
                    let item = expect!(entry[0].as_i64(),
                                       "failed to convert recipe {} {} {} item", i, what, j);
                    let count = expect!(entry[1].as_i64(),
                                        "failed to convert recipe {} {} {} count", i, what, j);
                    map.insert(item as ItemId, count as u8);
                }
                Ok(map)
            }

            let inputs = get_convert!(recipe, "inputs", as_array,
                                      "for recipe {}", i);
            let inputs = try!(build_map(&**inputs, "input", i));

            let outputs = get_convert!(recipe, "outputs", as_array,
                                       "for recipe {}", i);
            let outputs = try!(build_map(&**outputs, "input", i));

            recipes.push(Recipe {
                name: name.to_owned(),
                inputs: inputs,
                outputs: outputs,
                station: station,
            });
            name_to_id.insert(name.to_owned(), i as RecipeId); 
        }

        Ok(RecipeData {
            recipes: recipes,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    pub fn recipe(&self, id: RecipeId) -> &Recipe {
        &self.recipes[id as usize]
    }

    pub fn get_recipe(&self, id: RecipeId) -> Option<&Recipe> {
        self.recipes.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> RecipeId {
        self.find_id(name).unwrap_or_else(|| panic!("unknown recipe id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<RecipeId> {
        self.name_to_id.get(name).map(|&x| x)
    }
}
