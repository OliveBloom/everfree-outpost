use std::prelude::v1::*;
use std::mem;
use std::slice;
use std::str;

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



struct FileHeader {
    minor: u16,
    major: u16,
    num_sections: u32,
    _reserved0: u32,
    _reserved1: u32,
}

struct SectionHeader {
    name: [u8; 8],
    offset: u32,
    item_size: u16,
    item_count: u16,
}

const SUPPORTED_VERSION: (u16, u16) = (0, 1);

macro_rules! gen_data {
    ($($name:ident ($sect_name:pat): $ty:ty,)*) => {
        pub struct Data {
            raw: Box<[u8]>,
            strings: *const str,

            $( $name: *const [$ty], )*
        }

        impl Data {
            pub fn new(raw: Box<[u8]>) -> Data {
                let mut strings = None;
                $( let mut $name: Option<*const [$ty]> = None; )*

                unsafe {
                    let ptr = raw.as_ptr();
                    assert!(ptr as usize & 7 == 0,
                            "raw data allocation must be 8-byte aligned");

                    let header = &*(ptr as *const FileHeader);
                    let version = (header.major, header.minor);
                    assert!(version == SUPPORTED_VERSION,
                            "unsupported data file version (got {:?}, need {:?}",
                            version, SUPPORTED_VERSION);

                    let sections = slice::from_raw_parts(ptr.offset(16) as *const SectionHeader,
                                                         header.num_sections as usize);

                    for s in sections {
                        match &s.name {
                            b"Strings\0" => {
                                assert!(s.item_size == 1);
                                let bytes = slice::from_raw_parts(ptr.offset(s.offset as isize),
                                                                  s.item_count as usize);
                                let s = str::from_utf8(bytes).unwrap();
                                strings = Some(s);
                            },

                            $(
                                $sect_name => {
                                    assert!(s.item_size as usize == mem::size_of::<$ty>());
                                    $name = Some(slice::from_raw_parts(
                                        ptr.offset(s.offset as isize) as *const $ty,
                                        s.item_count as usize));
                                },
                            )*

                            _ => {
                                warn!("unknown data section: {:?}", s.name);
                            },
                        }
                    }
                }

                Data {
                    raw: raw,
                    strings: strings.expect(
                        concat!("missing section: ", stringify!(b"Strings\0"))),
                    $( $name: $name.expect(
                        concat!("missing section: ", stringify!($sect_name))), )*
                }
            }

            pub fn strings<'a>(&'a self) -> &'a str {
                unsafe { &*self.strings }
            }

            $(
                pub fn $name<'a>(&'a self) -> &'a [$ty] {
                    unsafe { &*self.$name }
                }
            )*
        }
    };
}

gen_data! {
    blocks (b"Blocks\0\0"): BlockData,
    raw_items (b"Items\0\0\0"): RawItemDef,

    templates (b"StrcDefs"): StructureTemplate,
    template_parts (b"StrcPart"): TemplatePart,
    template_verts (b"StrcVert"): TemplateVertex,
    template_shapes (b"StrcShap"): Shape,

    animations (b"SprtAnim"): Animation,
    sprite_layers (b"SprtLayr"): SpriteLayer,
    sprite_graphics (b"SprtGrfx"): SpriteGraphics,

    pony_layer_table (b"XPonLayr"): u8,
    physics_anim_table (b"XPhysAnm"): [u16; 8],
    anim_dir_table (b"XAnimDir"): u8,
}

impl Data {
    pub fn template_size(&self, template_id: u32) -> V3 {
        let t = &self.templates()[template_id as usize];
        util::unpack_v3(t.size)
    }

    pub fn template_shape(&self, template_id: u32) -> &[Shape] {
        let t = &self.templates()[template_id as usize];
        let base = t.shape_idx as usize;
        let size = util::unpack_v3(t.size);
        let volume = (size.x * size.y * size.z) as usize;
        &self.template_shapes()[base .. base + volume]
    }

    fn make_item_def<'a>(&'a self, raw: &'a RawItemDef) -> ItemDef<'a> {
        ItemDef {
            def: raw,
            strs: self.strings(),
        }
    }

    pub fn item_def(&self, id: u16) -> ItemDef {
        self.make_item_def(&self.raw_items()[id as usize])
    }

    pub fn find_item_id(&self, name: &str) -> Option<u16> {
        for (i, raw) in self.raw_items().iter().enumerate() {
            let def = self.make_item_def(raw);
            if name == def.name() {
                return Some(i as u16);
            }
        }
        None
    }


    pub fn template(&self, id: u32) -> &StructureTemplate {
        &self.templates()[id as usize]
    }


    pub fn animation(&self, id: u16) -> &Animation {
        &self.animations()[id as usize]
    }

    pub fn sprite_layer(&self, id: u8) -> &SpriteLayer {
        &self.sprite_layers()[id as usize]
    }

    pub fn sprite_graphics_item(&self, id: u16) -> &SpriteGraphics {
        &self.sprite_graphics()[id as usize]
    }
}

/*

pub struct Data {
    raw: Box<[u8]>,

    strings: *const str,

    blocks: *const [BlockData],

    item_defs: *const [RawItemDef],

    templates: *const [StructureTemplate],
    template_parts: *const [TemplatePart],
    template_verts: *const [TemplateVertex],

    animations: *const [Animation],
    sprite_layers: *const [SpriteLayer],
    sprite_graphics: *const [SpriteGraphics],

    pony_layer_table: *const [u8],
    physics_anim_table: *const [[u16; 8]],
    anim_dir_table: *const [u8],
}

impl Data {
    pub fn new(raw: Box
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
*/
