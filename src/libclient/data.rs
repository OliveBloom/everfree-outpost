use std::prelude::v1::*;

use physics::Shape;

use graphics::types::{BlockData, StructureTemplate, TemplatePart, TemplateVertex};


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
}
