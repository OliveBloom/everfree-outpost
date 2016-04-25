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

pub struct DayNightPhase {
    pub start_time: u16,
    pub end_time: u16,
    pub start_color: u8,
    pub end_color: u8,
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

    day_night_phases (b"DyNtPhas"): DayNightPhase,
    day_night_colors (b"DyNtColr"): (u8, u8, u8),

    pony_layer_table (b"XPonLayr"): u8,
    physics_anim_table (b"XPhysAnm"): [u16; 8],
    anim_dir_table (b"XAnimDir"): u8,
    special_anims (b"XSpcAnim"): u16,
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


    pub fn day_night_phase(&self, idx: u8) -> &DayNightPhase {
        &self.day_night_phases()[idx as usize]
    }


    pub fn anim_dir(&self, anim: u16) -> Option<u8> {
        match self.anim_dir_table().get(anim as usize) {
            None | Some(&255) => None,
            Some(&x) => Some(x),
        }
    }

    pub fn default_anim(&self) -> u16 {
        self.special_anims()[0]
    }

    pub fn editor_anim(&self) -> u16 {
        self.special_anims()[1]
    }
}
