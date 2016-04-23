use std::prelude::v1::*;
use std::mem;
use std::slice;

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
    extras: Box<[u8]>,
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
               extras: Box<[u8]>) -> Data {
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
            extras: extras,
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

    fn extras_header(&self) -> (u32, u32) {
        let hdr = unsafe { cast_bytes(&self.extras[0 .. 8]) };
        assert!(hdr.len() == 1);
        hdr[0]
    }

    fn raw_extra(&self, i: usize) -> &[u8] {
        let (index_size, _) = self.extras_header();
        let byte_len = mem::size_of::<(u32, u32)>() * index_size as usize;
        let index: &[(u32, u32)] = unsafe { cast_bytes(&self.extras[8 .. 8 + byte_len]) };
        let (start, end) = index[i];
        &self.extras[start as usize .. end as usize]
    }

    pub fn pony_layer_table(&self) -> &[u8] {
        self.raw_extra(PONY_LAYER_TABLE_IDX)
    }

    pub fn physics_anim_table(&self) -> &[[u16; 8]] {
        unsafe { cast_bytes(self.raw_extra(PHYSICS_ANIM_TABLE_IDX)) }
    }

    pub fn anim_dir_table(&self) -> &[u8] {
        self.raw_extra(ANIM_DIR_TABLE_IDX)
    }
}

const PONY_LAYER_TABLE_IDX: usize =     0;
const PHYSICS_ANIM_TABLE_IDX: usize =   1;
const ANIM_DIR_TABLE_IDX: usize =       2;

unsafe fn cast_bytes<T>(b: &[u8]) -> &[T] {
    slice::from_raw_parts(b.as_ptr() as *const T,
                          b.len() / mem::size_of::<T>())
}
