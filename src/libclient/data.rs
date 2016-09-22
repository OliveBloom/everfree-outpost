use std::prelude::v1::*;
use std::hash::{Hash, Hasher, SipHasher};
use std::mem;
use std::ops::Deref;
use std::slice;
use std::str;

use physics::Shape;
use physics::v3::V3;
use common_types::{BlockFlags, BlockId};

use graphics::types::{StructureTemplate, TemplatePart, TemplateVertex};
use util;


/// Tile numbers used to display a particular block.
#[derive(Clone, Copy)]
pub struct BlockDef {
    // 0
    pub front: u16,
    pub back: u16,
    pub top: u16,
    pub bottom: u16,

    // 8
    pub light_color: (u8, u8, u8),
    pub _pad1: u8,
    pub light_radius: u16,
    pub raw_flags: u16,

    // 16
}

impl BlockDef {
    pub fn tile(&self, side: usize) -> u16 {
        match side {
            0 => self.front,
            1 => self.back,
            2 => self.top,
            3 => self.bottom,
            _ => panic!("invalid side number"),
        }
    }

    pub fn flags(&self) -> BlockFlags {
        BlockFlags::from_bits_truncate(self.raw_flags)
    }
}


pub struct RawItemDef {
    pub name_off: usize,
    pub name_len: usize,
    pub ui_name_off: usize,
    pub ui_name_len: usize,
    pub desc_off: usize,
    pub desc_len: usize,
}

pub struct ItemDef<'a> {
    def: &'a RawItemDef,
    data: &'a Data,
}

impl<'a> ItemDef<'a> {
    pub fn name(&self) -> &'a str {
        self.data.string_slice(self.def.name_off, self.def.name_len)
    }

    pub fn ui_name(&self) -> &'a str {
        self.data.string_slice(self.def.ui_name_off, self.def.ui_name_len)
    }

    pub fn desc(&self) -> &'a str {
        self.data.string_slice(self.def.desc_off, self.def.desc_len)
    }
}


pub struct RawRecipeDef {
    pub ui_name_off: usize,
    pub ui_name_len: usize,
    pub inputs_off: u16,
    pub inputs_len: u16,
    pub outputs_off: u16,
    pub outputs_len: u16,
    pub ability: u16,
    _pad0: u16,
    pub station: u32,
}

pub struct RecipeDef<'a> {
    def: &'a RawRecipeDef,
    data: &'a Data,
}

impl<'a> RecipeDef<'a> {
    pub fn ui_name(&self) -> &'a str {
        self.data.string_slice(self.def.ui_name_off, self.def.ui_name_len)
    }

    pub fn inputs(&self) -> &'a [RecipeItem] {
        let off = self.def.inputs_off as usize;
        let len = self.def.inputs_len as usize;
        self.data.recipe_item_slice(off, len)
    }

    pub fn outputs(&self) -> &'a [RecipeItem] {
        let off = self.def.outputs_off as usize;
        let len = self.def.outputs_len as usize;
        self.data.recipe_item_slice(off, len)
    }
}

impl<'a> Deref for RecipeDef<'a> {
    type Target = RawRecipeDef;
    fn deref(&self) -> &RawRecipeDef {
        self.def
    }
}

pub struct RecipeItem {
    pub item: u16,
    pub quantity: u16,
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
    len: u32,
}

const SUPPORTED_VERSION: (u16, u16) = (2, 0);


/// Parameters for the CHD perfect hash function.
pub struct ChdParams<T> {
    /// `m` - the modulus = number of buckets for the main hash table.
    m: u32,
    /// `l_i` - the key used for the `i`^th bucket of the intermediate table.
    l: [T],
}


unsafe trait Section {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const Self;
}

// TODO: should be `T: Bytes`
unsafe impl<T> Section for [T] {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const [T] {
        slice::from_raw_parts(ptr as *const T,
                              len / mem::size_of::<T>())
    }
}

unsafe impl Section for str {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const str {
        let bytes = <[u8] as Section>::from_bytes(ptr, len);
        str::from_utf8(&*bytes).unwrap()
    }
}

// TODO: should be `T: Bytes`
unsafe impl<T> Section for ChdParams<T> {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const ChdParams<T> {
        let dummy_slice: *const [u8] = slice::from_raw_parts(4096 as *const u8, 0);
        let dummy_params: *const ChdParams<T> = mem::transmute(dummy_slice);
        let offset = mem::size_of_val(&*dummy_params);

        let adj_len = len - offset;
        let slice = slice::from_raw_parts(ptr as *const T,
                                          adj_len / mem::size_of::<T>());
        let params: &ChdParams<T> = mem::transmute(slice);
        // Hash table moduli must be powers of two.
        fn is_power_of_two(x: usize) -> bool { x & (x - 1) == 0 }
        assert!(is_power_of_two(params.m as usize));
        assert!(is_power_of_two(params.l.len()));
        params
    }
}


macro_rules! gen_data {
    ($($name:ident ($sect_name:pat): $ty:ty,)*) => {
        pub struct Data {
            // `raw` is never referenced directly, but holds ownership for the other fields.
            #[allow(dead_code)]
            raw: Box<[u8]>,

            $( $name: *const $ty, )*
        }

        impl Data {
            pub fn new(raw: Box<[u8]>) -> Data {
                $( let mut $name: Option<*const $ty> = None; )*

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
                            $(
                                $sect_name => {
                                    $name = Some(<$ty as Section>::from_bytes(
                                        ptr.offset(s.offset as isize),
                                        s.len as usize));
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
                    $( $name: $name.expect(
                        concat!("missing section: ", stringify!($sect_name))), )*
                }
            }

            $(
                pub fn $name<'a>(&'a self) -> &'a $ty {
                    unsafe { &*self.$name }
                }
            )*
        }
    };
}

gen_data! {
    strings (b"Strings\0"): str,

    blocks (b"Blocks\0\0"): [BlockDef],
    raw_items (b"Items\0\0\0"): [RawItemDef],

    templates (b"StrcDefs"): [StructureTemplate],
    template_parts (b"StrcPart"): [TemplatePart],
    template_verts (b"StrcVert"): [TemplateVertex],
    // TODO: need a check to ensure all the flags are valid (under BlockFlags::all())
    template_shapes (b"StrcShap"): [BlockFlags],

    animations (b"SprtAnim"): [Animation],
    sprite_layers (b"SprtLayr"): [SpriteLayer],
    sprite_graphics (b"SprtGrfx"): [SpriteGraphics],

    recipes (b"RcpeDefs"): [RawRecipeDef],
    recipe_items (b"RcpeItms"): [RecipeItem],

    day_night_phases (b"DyNtPhas"): [DayNightPhase],
    day_night_colors (b"DyNtColr"): [(u8, u8, u8)],

    pony_layer_table (b"XPonLayr"): [u8],
    physics_anim_table (b"XPhysAnm"): [[u16; 8]],
    anim_dir_table (b"XAnimDir"): [u8],
    special_anims (b"XSpcAnim"): [u16],
    special_layers (b"XSpcLayr"): [u8],
    special_graphics (b"XSpcGrfx"): [u16],

    index_table_items (b"IxTbItem"): [u16],
    index_params_items (b"IxPrItem"): ChdParams<u16>,
}

/// Apply the CHD-derived perfect hash function to obtain the index of the bucket assigned to the
/// indicated key.
fn chd_lookup(key: &str, table: &[u16], params: &ChdParams<u16>) -> u16 {
    let mut h1 = SipHasher::new_with_keys(0x123456, 0xfedcba);
    key.hash(&mut h1);
    // params.l.len() is a power of two, so use & instead of %
    let idx1 = h1.finish() as usize & (params.l.len() - 1);
    let l = params.l[idx1] as u64;

    let mut h2 = SipHasher::new_with_keys(0x123456 + l, 0xfedcba - l);
    key.hash(&mut h2);
    let idx2 = h2.finish() as usize & (params.m as usize - 1);
    let result = table.get(idx2).map_or(0xffff, |&x| x);

    result
}

impl Data {
    fn string_slice(&self, off: usize, len: usize) -> &str {
        &self.strings()[off .. off + len]
    }

    fn recipe_item_slice(&self, off: usize, len: usize) -> &[RecipeItem] {
        &self.recipe_items()[off .. off + len]
    }


    // TODO: remove these template_x methods, use template(id).field instead
    pub fn template_size(&self, template_id: u32) -> V3 {
        let t = &self.templates()[template_id as usize];
        util::unpack_v3(t.size)
    }

    pub fn template_shape(&self, template_id: u32) -> &[BlockFlags] {
        let t = &self.templates()[template_id as usize];
        let base = t.shape_idx as usize;
        let size = util::unpack_v3(t.size);
        let volume = (size.x * size.y * size.z) as usize;
        &self.template_shapes()[base .. base + volume]
    }


    pub fn block(&self, id: BlockId) -> &BlockDef {
        &self.blocks()[id as usize]
    }


    fn make_item_def<'a>(&'a self, raw: &'a RawItemDef) -> ItemDef<'a> {
        ItemDef {
            def: raw,
            data: self,
        }
    }

    pub fn item_def(&self, id: u16) -> ItemDef {
        self.make_item_def(&self.raw_items()[id as usize])
    }

    pub fn find_item_id(&self, name: &str) -> Option<u16> {
        let idx = chd_lookup(name, self.index_table_items(), self.index_params_items());
        if (idx as usize) < self.raw_items().len() && self.item_def(idx).name() == name {
            Some(idx)
        } else {
            None
        }
    }


    pub fn recipe(&self, id: u16) -> RecipeDef {
        RecipeDef {
            def: &self.recipes()[id as usize],
            data: self,
        }
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

    pub fn activity_bubble_graphics(&self) -> u16 {
        self.special_graphics()[0]
    }

    pub fn activity_layer_id(&self) -> u8 {
        self.special_layers()[0]
    }

    pub fn activity_layer(&self) -> &SpriteLayer {
        self.sprite_layer(self.activity_layer_id())
    }

    pub fn activity_none_anim(&self) -> u16 {
        self.special_anims()[2]
    }
}
