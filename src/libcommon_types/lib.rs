#![crate_name = "common_types"]

#![cfg_attr(asmjs, no_std)]
#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(asmjs)] use std::prelude::v1::*;

use std::u8;


// Typedef IDs.  These are used to identify game data elements.

pub type AnimId = u16;
pub type BlockId = u16;
pub type ItemId = u16;
pub type RecipeId = u16;
pub type TileId = u16;
pub type TemplateId = u32;
pub type SlotId = u8;

// Well-known typedef ID values.
pub const EMPTY_BLOCK: BlockId = 0;
pub const PLACEHOLDER_BLOCK: BlockId = 1;
pub const NO_SLOT: SlotId = u8::MAX;
pub const NO_ITEM: ItemId = 0;


// Newtype IDs.  These are used to identify game objects (parts of the World).

#[macro_export]
macro_rules! mk_id_newtypes {
    ( $($name:ident($inner:ty);)* ) => {
        $(
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
            pub struct $name(pub $inner);

            impl $name {
                pub fn unwrap(self) -> $inner {
                    let $name(x) = self;
                    x
                }
            }

            impl From<$name> for usize {
                fn from(this: $name) -> usize {
                    this.unwrap() as usize
                }
            }

            impl From<$name> for $inner {
                fn from(this: $name) -> $inner {
                    this.unwrap()
                }
            }

            impl From<$inner> for $name {
                fn from(this: $inner) -> $name {
                    $name(this)
                }
            }
        )*
    };
}

mk_id_newtypes! {
    ClientId(u16);
    EntityId(u32);
    InventoryId(u32);
    PlaneId(u32);
    TerrainChunkId(u32);
    StructureId(u32);
}

