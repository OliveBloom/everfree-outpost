use std::mem;
use std::ops::Deref;
use std::slice;
use std::str;

use libserver_types::*;
use libcommon_data::{Section, ChdParams, chd_lookup};
use libcommon_util::Bytes;


#[derive(Clone, Copy, Debug)]
pub struct Block {
    name_off: u32,
    name_len: u32,
    flags: u16,
}
unsafe impl Bytes for Block {}

impl Block {
    pub fn flags(&self) -> BlockFlags {
        BlockFlags::from_bits_truncate(self.flags)
    }
}

#[derive(Clone, Copy)]
pub struct BlockRef<'a> {
    obj: &'a Block,
    base: &'a Data,
}

impl<'a> Deref for BlockRef<'a> {
    type Target = Block;
    fn deref(&self) -> &Block {
        self.obj
    }
}

impl<'a> BlockRef<'a> {
    pub fn name(&self) -> &'a str {
        self.base.string_slice(self.obj.name_off, self.obj.name_len)
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Item {
    name_off: u32,
    name_len: u32,
}
unsafe impl Bytes for Item {}

#[derive(Clone, Copy)]
pub struct ItemRef<'a> {
    obj: &'a Item,
    base: &'a Data,
}

impl<'a> Deref for ItemRef<'a> {
    type Target = Item;
    fn deref(&self) -> &Item {
        self.obj
    }
}

impl<'a> ItemRef<'a> {
    pub fn name(&self) -> &'a str {
        self.base.string_slice(self.obj.name_off, self.obj.name_len)
    }
}



#[derive(Clone, Copy, Debug)]
pub struct RecipeItem {
    pub item: ItemId,
    pub quantity: u16,
}
unsafe impl Bytes for RecipeItem {}

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    name_off: u32,
    name_len: u32,
    inputs_off: u32,
    inputs_len: u32,
    outputs_off: u32,
    outputs_len: u32,
    pub ability: ItemId,
    pub station: TemplateId,
}
unsafe impl Bytes for Recipe {}

#[derive(Clone, Copy)]
pub struct RecipeRef<'a> {
    obj: &'a Recipe,
    base: &'a Data,
}

impl<'a> Deref for RecipeRef<'a> {
    type Target = Recipe;
    fn deref(&self) -> &Recipe {
        self.obj
    }
}

impl<'a> RecipeRef<'a> {
    pub fn name(&self) -> &'a str {
        self.base.string_slice(self.obj.name_off, self.obj.name_len)
    }

    pub fn inputs(&self) -> &'a [RecipeItem] {
        let off = self.inputs_off as usize;
        let len = self.inputs_len as usize;
        &self.base.recipe_items()[off .. off + len]
    }

    pub fn outputs(&self) -> &'a [RecipeItem] {
        let off = self.outputs_off as usize;
        let len = self.outputs_len as usize;
        &self.base.recipe_items()[off .. off + len]
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Template {
    name_off: u32,
    name_len: u32,
    pub size: V3,
    shape_off: u32,
    shape_len: u32,
    pub layer: u8,
}
unsafe impl Bytes for Template {}

#[derive(Clone, Copy)]
pub struct TemplateRef<'a> {
    obj: &'a Template,
    base: &'a Data,
}

impl<'a> Deref for TemplateRef<'a> {
    type Target = Template;
    fn deref(&self) -> &Template {
        self.obj
    }
}

impl<'a> TemplateRef<'a> {
    pub fn name(&self) -> &'a str {
        self.base.string_slice(self.obj.name_off, self.obj.name_len)
    }

    pub fn shape(&self) -> &'a [BlockFlags] {
        let off = self.shape_off as usize;
        let len = self.shape_len as usize;
        &self.base.template_shapes().0[off .. off + len]
    }
}



#[derive(Clone, Copy, Debug)]
pub struct Animation {
    name_off: u32,
    name_len: u32,
    pub framerate: u8,
    pub length: u8,
}
unsafe impl Bytes for Animation {}

#[derive(Clone, Copy)]
pub struct AnimationRef<'a> {
    obj: &'a Animation,
    base: &'a Data,
}

impl<'a> Deref for AnimationRef<'a> {
    type Target = Animation;
    fn deref(&self) -> &Animation {
        self.obj
    }
}

impl<'a> AnimationRef<'a> {
    pub fn name(&self) -> &'a str {
        self.base.string_slice(self.obj.name_off, self.obj.name_len)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteLayer {
    name_off: u32,
    name_len: u32,
}
unsafe impl Bytes for SpriteLayer {}

#[derive(Clone, Copy)]
pub struct SpriteLayerRef<'a> {
    obj: &'a SpriteLayer,
    base: &'a Data,
}

impl<'a> Deref for SpriteLayerRef<'a> {
    type Target = SpriteLayer;
    fn deref(&self) -> &SpriteLayer {
        self.obj
    }
}

impl<'a> SpriteLayerRef<'a> {
    pub fn name(&self) -> &'a str {
        self.base.string_slice(self.obj.name_off, self.obj.name_len)
    }
}


pub struct BlockFlagsArray([BlockFlags]);

unsafe impl Section for BlockFlagsArray {
    unsafe fn from_bytes(ptr: *const u8, len: usize) -> *const BlockFlagsArray {
        assert!(mem::size_of::<u16>() == mem::size_of::<BlockFlags>());
        let raw = slice::from_raw_parts(ptr as *const u16,
                                        len / mem::size_of::<u16>());
        assert!(raw.iter().all(|&x| x & !BlockFlags::all().bits() == 0),
                "found invalid bits in BlockFlags array");
        mem::transmute(raw as *const [u16])
    }
}


gen_data! {
    version = (2, 0);

    strings (b"Strings\0"): str,

    blocks (b"Blocks\0\0"): [Block],
    block_params (b"IxPrBlck"): ChdParams<u16>,
    block_table (b"IxTbBlck"): [u16],

    items (b"Items\0\0\0"): [Item],
    item_params (b"IxPrItem"): ChdParams<u16>,
    item_table (b"IxTbItem"): [u16],

    recipes (b"RcpeDefs"): [Recipe],
    recipe_items (b"RcpeItms"): [RecipeItem],
    recipe_params (b"IxPrRcpe"): ChdParams<u16>,
    recipe_table (b"IxTbRcpe"): [u16],

    templates (b"StrcDefs"): [Template],
    template_shapes (b"StrcShap"): BlockFlagsArray,
    template_params (b"IxPrStrc"): ChdParams<u16>,
    template_table (b"IxTbStrc"): [u16],

    animations (b"SprtAnim"): [Animation],
    animation_params (b"IxPrAnim"): ChdParams<u16>,
    animation_table (b"IxTbAnim"): [u16],
}

impl Data {
    fn string_slice(&self, off: u32, len: u32) -> &str {
        let off = off as usize;
        let len = len as usize;
        &self.strings()[off .. off + len]
    }


    pub fn get_block(&self, id: BlockId) -> Option<BlockRef> {
        self.blocks().get(id as usize)
            .map(|obj| BlockRef { obj: obj, base: self })
    }

    pub fn block(&self, id: BlockId) -> BlockRef {
        self.get_block(id)
            .unwrap_or_else(|| panic!("unknown block id: {}", id))
    }

    pub fn get_block_id(&self, name: &str) -> Option<BlockId> {
        if let Some(id) = chd_lookup(name, self.block_table(), self.block_params()) {
            let id = id as BlockId;
            if self.get_block(id).map_or(false, |b| b.name() == name) {
                return Some(id);
            }
        }
        None
    }

    pub fn block_id(&self, name: &str) -> BlockId {
        self.get_block_id(name)
            .unwrap_or_else(|| panic!("unknown block name: {:?}", name))
    }


    pub fn get_item(&self, id: ItemId) -> Option<ItemRef> {
        self.items().get(id as usize)
            .map(|obj| ItemRef { obj: obj, base: self })
    }

    pub fn item(&self, id: ItemId) -> ItemRef {
        self.get_item(id)
            .unwrap_or_else(|| panic!("unknown item id: {}", id))
    }

    pub fn get_item_id(&self, name: &str) -> Option<ItemId> {
        if let Some(id) = chd_lookup(name, self.item_table(), self.item_params()) {
            let id = id as ItemId;
            if self.get_item(id).map_or(false, |b| b.name() == name) {
                return Some(id);
            }
        }
        None
    }

    pub fn item_id(&self, name: &str) -> ItemId {
        self.get_item_id(name)
            .unwrap_or_else(|| panic!("unknown item name: {:?}", name))
    }


    pub fn get_recipe(&self, id: RecipeId) -> Option<RecipeRef> {
        self.recipes().get(id as usize)
            .map(|obj| RecipeRef { obj: obj, base: self })
    }

    pub fn recipe(&self, id: RecipeId) -> RecipeRef {
        self.get_recipe(id)
            .unwrap_or_else(|| panic!("unknown recipe id: {}", id))
    }

    pub fn get_recipe_id(&self, name: &str) -> Option<RecipeId> {
        if let Some(id) = chd_lookup(name, self.recipe_table(), self.recipe_params()) {
            let id = id as RecipeId;
            if self.get_recipe(id).map_or(false, |b| b.name() == name) {
                return Some(id);
            }
        }
        None
    }

    pub fn recipe_id(&self, name: &str) -> RecipeId {
        self.get_recipe_id(name)
            .unwrap_or_else(|| panic!("unknown recipe name: {:?}", name))
    }


    pub fn get_template(&self, id: TemplateId) -> Option<TemplateRef> {
        self.templates().get(id as usize)
            .map(|obj| TemplateRef { obj: obj, base: self })
    }

    pub fn template(&self, id: TemplateId) -> TemplateRef {
        self.get_template(id)
            .unwrap_or_else(|| panic!("unknown template id: {}", id))
    }

    pub fn get_template_id(&self, name: &str) -> Option<TemplateId> {
        if let Some(id) = chd_lookup(name, self.template_table(), self.template_params()) {
            let id = id as TemplateId;
            if self.get_template(id).map_or(false, |b| b.name() == name) {
                return Some(id);
            }
        }
        None
    }

    pub fn template_id(&self, name: &str) -> TemplateId {
        self.get_template_id(name)
            .unwrap_or_else(|| panic!("unknown template name: {:?}", name))
    }


    pub fn get_animation(&self, id: AnimId) -> Option<AnimationRef> {
        self.animations().get(id as usize)
            .map(|obj| AnimationRef { obj: obj, base: self })
    }

    pub fn animation(&self, id: AnimId) -> AnimationRef {
        self.get_animation(id)
            .unwrap_or_else(|| panic!("unknown animation id: {}", id))
    }

    pub fn get_animation_id(&self, name: &str) -> Option<AnimId> {
        if let Some(id) = chd_lookup(name, self.animation_table(), self.animation_params()) {
            let id = id as AnimId;
            if self.get_animation(id).map_or(false, |b| b.name() == name) {
                return Some(id);
            }
        }
        None
    }

    pub fn animation_id(&self, name: &str) -> AnimId {
        self.get_animation_id(name)
            .unwrap_or_else(|| panic!("unknown animation name: {:?}", name))
    }
}
