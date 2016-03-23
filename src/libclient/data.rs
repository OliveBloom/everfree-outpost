use std::prelude::v1::*;

use graphics::structures;
use graphics::types::{BlockData, StructureTemplate, TemplatePart, TemplateVertex};


pub struct Data {
    pub template_data: structures::geom::TemplateData,
    pub block_data: Box<[BlockData]>,
}

impl Data {
    pub fn new(block_data: Box<[BlockData]>,
               templates: Box<[StructureTemplate]>,
               parts: Box<[TemplatePart]>,
               verts: Box<[TemplateVertex]>) -> Data {
        Data {
            template_data: structures::geom::TemplateData::new(templates, parts, verts),
            block_data: block_data,
        }
    }

    pub fn blocks_ptr(&self) -> *mut BlockData {
        self.block_data.as_ptr() as *mut _
    }

    pub fn templates_ptr(&self) -> *mut StructureTemplate {
        self.template_data.templates_ptr()
    }

    pub fn template_parts_ptr(&self) -> *mut TemplatePart {
        self.template_data.template_parts_ptr()
    }

    pub fn template_verts_ptr(&self) -> *mut TemplateVertex {
        self.template_data.template_verts_ptr()
    }
}
