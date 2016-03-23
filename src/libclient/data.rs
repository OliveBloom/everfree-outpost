use std::prelude::v1::*;

use physics::Shape;
use physics::v3::V3;

use graphics::types::{BlockData, StructureTemplate, TemplatePart, TemplateVertex};
use util;


pub struct Data {
    pub blocks: Box<[BlockData]>,
    pub templates: Box<[StructureTemplate]>,
    pub template_parts: Box<[TemplatePart]>,
    pub template_verts: Box<[TemplateVertex]>,
    pub template_shapes: Box<[Shape]>,
}

impl Data {
    pub fn new(blocks: Box<[BlockData]>,
               templates: Box<[StructureTemplate]>,
               template_parts: Box<[TemplatePart]>,
               template_verts: Box<[TemplateVertex]>,
               template_shapes: Box<[Shape]>) -> Data {
        Data {
            blocks: blocks,
            templates: templates,
            template_parts: template_parts,
            template_verts: template_verts,
            template_shapes: template_shapes,
        }
    }

    pub fn template_size(&self, template_id: u32) -> V3 {
        let t = &self.templates[template_id as usize];
        util::unpack_v3(t.size)
    }

    pub fn template_shape(&self, template_id: u32) -> &[Shape] {
        let t = &self.templates[template_id as usize];
        let base = t.shape_idx as usize;
        let size = util::unpack_v3(t.size);
        let volume = (size.x * size.y * size.z) as usize;
        &self.template_shapes[base .. base + volume]
    }
}
