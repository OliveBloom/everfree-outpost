#![crate_name = "common_types"]

#![cfg_attr(asmjs, no_std)]
#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(asmjs)] use std::prelude::v1::*;

#[macro_use] extern crate bitflags;

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


bitflags! {
    pub flags BlockFlags: u16 {
        // Cell parts. In a given cell, there can be at most one block for each part.
        //
        // There are two types of floor, called "floor" and "subfloor".  Terrain uses only
        // "subfloor", and structures use only "floor".  This allows player-built floors to overlap
        // the terrain.
        /// The block includes a subfloor (identical to a normal floor, but doesn't conflict).
        const B_SUBFLOOR =      0x0001,
        /// The block includdes a floor.
        const B_FLOOR =         0x0002,
        /// The block includes a solid component.
        const B_SOLID =         0x0004,
        const B_PART_MASK =     0x0007,

        /// If `B_SOLID` is set, these bits determine the actual shape for physics purposes.
        /// Otherwise, it's determined by the presence of `B_SUBFLOOR` and `B_FLOOR`.
        const B_SHAPE_MASK =    0xf000,

        /// The block stops vision.
        const B_OPAQUE =        0x0008,
        /// Despite having the correct shape, this block is not walkable.  Used for water/lava,
        /// which blocks both vertical flight (B_SUBFLOOR) and walking (B_NON_WALKABLE).
        const B_NON_WALKABLE =  0x0010,
        /// Makes the block "count" for purposes of player interaction hit-tests, and for
        /// determining whether two structures overlap in the same layer.
        ///
        /// Note that if `B_OCCUPIED` is unset for a structure block, then the remaining flags will
        /// have no effect.
        const B_OCCUPIED =      0x0020,
    }
}

impl BlockFlags {
    pub fn from_shape(s: Shape) -> BlockFlags {
        match s {
            Shape::Empty => BlockFlags::empty(),
            Shape::Floor => B_FLOOR,
            _ => B_SOLID | BlockFlags::from_bits_truncate((s as u8 as u16) << 12),
        }
    }

    pub fn shape(&self) -> Shape {
        if self.contains(B_SOLID) {
            let shape_num = ((*self & B_SHAPE_MASK).bits() >> 12) as usize;
            Shape::from_primitive(shape_num)
                .expect("invalid shape value in B_SHAPE_MASK")
        } else if self.contains(B_FLOOR) || self.contains(B_SUBFLOOR) {
            Shape::Floor
        } else {
            Shape::Empty
        }
    }

    pub fn occupied(&self) -> bool {
        self.contains(B_OCCUPIED)
    }

    pub fn parts(&self) -> BlockFlags {
        *self & B_PART_MASK
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Shape {
    Empty = 0,
    Floor = 1,
    Solid = 2,
    //RampE = 3,
    //RampW = 4,
    //RampS = 5,
    RampN = 6,
}

impl Shape {
    pub fn from_primitive(i: usize) -> Option<Shape> {
        use self::Shape::*;
        let s = match i {
            0 => Empty,
            1 => Floor,
            2 => Solid,
            6 => RampN,
            // TODO: add ramp variants once they are actually supported
            _ => return None,
        };
        Some(s)
    }

    pub fn is_ramp(&self) -> bool {
        use self::Shape::*;
        match *self {
            RampN => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self {
            Shape::Empty => true,
            _ => false,
        }
    }
}


