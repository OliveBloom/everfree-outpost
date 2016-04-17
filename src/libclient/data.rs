use std::prelude::v1::*;

use physics::Shape;
use physics::v3::V3;

use graphics::types::{BlockData, StructureTemplate, TemplatePart, TemplateVertex};
use util;


pub struct RawItemDef {
    pub name_off: usize,
    pub name_len: usize,
    pub ui_name_off: usize,
    pub ui_name_len: usize,
}

pub struct ItemDef<'a> {
    def: &'a RawItemDef,
    strs: &'a str,
}

impl<'a> ItemDef<'a> {
    fn slice(&self, off: usize, len: usize) -> &'a str {
        &self.strs[off .. off + len]
    }

    pub fn name(&self) -> &'a str {
        self.slice(self.def.name_off, self.def.name_len)
    }

    pub fn ui_name(&self) -> &'a str {
        self.slice(self.def.ui_name_off, self.def.ui_name_len)
    }
}


pub struct Animation {
    pub local_id: u16,
    pub framerate: u8,
    pub length: u8,
}

pub struct SpriteLayer {
    pub gfx_start: u16,
    pub gfx_count: u16,
}

pub struct SpriteGraphics {
    pub src_offset: (u16, u16),
    pub dest_offset: (u16, u16),
    pub size: (u16, u16),
    pub sheet: u8,
    pub mirror: u8,
}


pub struct Data {
    pub blocks: Box<[BlockData]>,
    item_defs: Box<[RawItemDef]>,
    item_strs: Box<str>,
    pub templates: Box<[StructureTemplate]>,
    pub template_parts: Box<[TemplatePart]>,
    pub template_verts: Box<[TemplateVertex]>,
    pub template_shapes: Box<[Shape]>,
    pub animations: Box<[Animation]>,
    pub sprite_layers: Box<[SpriteLayer]>,
    pub sprite_graphics: Box<[SpriteGraphics]>,
    pub pony_layer_table: Box<[u8]>,
}

impl Data {
    pub fn new(blocks: Box<[BlockData]>,
               item_defs: Box<[RawItemDef]>,
               item_strs: Box<str>,
               templates: Box<[StructureTemplate]>,
               template_parts: Box<[TemplatePart]>,
               template_verts: Box<[TemplateVertex]>,
               template_shapes: Box<[Shape]>,
               animations: Box<[Animation]>,
               sprite_layers: Box<[SpriteLayer]>,
               sprite_graphics: Box<[SpriteGraphics]>,
               pony_layer_table: Box<[u8]>) -> Data {
        Data {
            blocks: blocks,
            item_strs: item_strs,
            item_defs: item_defs,
            templates: templates,
            template_parts: template_parts,
            template_verts: template_verts,
            template_shapes: template_shapes,
            animations: animations,
            sprite_layers: sprite_layers,
            sprite_graphics: sprite_graphics,
            pony_layer_table: pony_layer_table,
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

    fn make_item_def<'a>(&'a self, raw: &'a RawItemDef) -> ItemDef<'a> {
        ItemDef {
            def: raw,
            strs: &self.item_strs,
        }
    }

    pub fn item_def(&self, id: u16) -> ItemDef {
        self.make_item_def(&self.item_defs[id as usize])
    }

    pub fn find_item_id(&self, name: &str) -> Option<u16> {
        for (i, raw) in self.item_defs.iter().enumerate() {
            let def = self.make_item_def(raw);
            if name == def.name() {
                return Some(i as u16);
            }
        }
        None
    }
}
