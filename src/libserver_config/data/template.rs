use std::collections::HashMap;
use rustc_serialize::json::Json;

use libserver_types::*;

use super::ParseError;

pub struct StructureTemplate {
    pub name: String,
    pub size: V3,
    pub shape: Vec<BlockFlags>,
    pub layer: u8,
}

pub struct StructureTemplates {
    templates: Vec<StructureTemplate>,
    name_to_id: HashMap<String, TemplateId>,
}

impl StructureTemplates {
    pub fn from_json(json: Json) -> Result<StructureTemplates, ParseError> {
        let templates = expect!(json.as_array(),
                                "found non-array at top level");

        let mut by_id = Vec::with_capacity(templates.len());
        let mut name_to_id = HashMap::new();

        for (i, template) in templates.iter().enumerate() {
            let name = get_convert!(template, "name", as_string,
                                    "for template {}", i);
            let size_arr = get_convert!(template, "size", as_array,
                                        "for template {} ({})", i, name);
            let shape_arr = get_convert!(template, "shape", as_array,
                                         "for template {} ({})", i, name);
            let layer = get_convert!(template, "layer", as_i64,
                                     "for template {} ({})", i, name);

            if size_arr.len() != 3 {
                return fail!("wrong number of elements in templates[{}].size ({})",
                             i, name);
            }

            let size_x = expect!(size_arr[0].as_i64(),
                                 "non-integer in templates[{}].size ({})", i, name);
            let size_y = expect!(size_arr[1].as_i64(),
                                 "non-integer in templates[{}].size ({})", i, name);
            let size_z = expect!(size_arr[2].as_i64(),
                                 "non-integer in templates[{}].size ({})", i, name);

            let size = V3::new(size_x as i32,
                               size_y as i32,
                               size_z as i32);

            let mut shape = Vec::with_capacity(shape_arr.len());
            for (j, shape_json) in shape_arr.iter().enumerate() {
                let shape_disr = expect!(shape_json.as_i64(),
                                         "non-integer at templates[{}].shape[{}] ({})",
                                         i, j, name);
                let shape_enum = expect!(Shape::from_primitive(shape_disr as usize),
                                         "invalid shape {} at templates[{}].shape[{}] ({})",
                                         shape_disr, i, j, name);
                shape.push(BlockFlags::from_shape(shape_enum) | B_OCCUPIED);
            }

            info!("parsed template: {}", name);
            by_id.push(StructureTemplate {
                name: name.to_owned(),
                size: size,
                shape: shape,
                layer: layer as u8,
            });
            name_to_id.insert(name.to_owned(), i as TemplateId);
        }

        Ok(StructureTemplates {
            templates: by_id,
            name_to_id: name_to_id,
        })
    }

    pub fn len(&self) -> usize {
        self.templates.len()
    }

    pub fn template(&self, id: TemplateId) -> &StructureTemplate {
        self.get_template(id).unwrap()
    }

    pub fn get_template(&self, id: TemplateId) -> Option<&StructureTemplate> {
        self.templates.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> TemplateId {
        self.find_id(name).unwrap_or_else(|| panic!("unknown structure template id: {}", name))
    }

    pub fn find_id(&self, name: &str) -> Option<TemplateId> {
        self.name_to_id.get(name).map(|&x| x)
    }

    pub fn get_by_id(&self, name: &str) -> &StructureTemplate {
        self.template(self.get_id(name))
    }
}
