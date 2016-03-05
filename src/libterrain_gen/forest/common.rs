use std::iter;

use libserver_types::*;

use cache::{self, Cache};
use forest::context::Context;


pub trait GridLike {
    type Elem: Copy;
    fn spacing() -> i32;
    fn size() -> i32;
    fn bounds() -> Region<V2> { Region::new(scalar(0), scalar(Self::size())) }
    fn get(&self, offset: V2) -> Self::Elem;
}

macro_rules! define_grid {
    ($Grid:ident : $T:ty; $SPACING:expr) => {
        define_grid!($Grid: $T; $SPACING; $SPACING);
    };
    ($Grid:ident : $T:ty; $SPACING:expr; + $extra:expr) => {
        define_grid!($Grid: $T; $SPACING; $SPACING + $extra);
    };
    ($Grid:ident : $T:ty; $SPACING:expr; $SIZE:expr) => {
        pub struct $Grid {
            pub data: [$T; $SIZE * $SIZE],
        }

        impl ::cache::Summary for $Grid {
            fn alloc() -> Box<$Grid> {
                use std::mem;
                // TODO: safe mem::zeroed() wrapper for Bytes
                // This is fine for now because the code below requires $T: Bytes
                Box::new(unsafe { mem::zeroed() })
            }

            fn write_to(&self, mut f: File) -> ::std::io::Result<()> {
                use libserver_util::bytes::WriteBytes;
                f.write_bytes_slice(&self.data)
            }

            fn read_from(mut f: File) -> ::std::io::Result<Box<$Grid>> {
                use libserver_util::bytes::ReadBytes;
                let mut result = $Grid::alloc();
                try!(f.read_bytes_slice(&mut result.data));
                Ok(result)
            }
        }

        impl ::forest::common::GridLike for $Grid {
            type Elem = $T;

            fn spacing() -> i32 { $SPACING as i32 }
            fn size() -> i32 { $SIZE as i32 }

            fn get(&self, offset: V2) -> $T {
                use forest::common::GridLike;
                self.data[Self::bounds().index(offset)]
            }
        }
    };
}


pub trait PointsLike {
    type Elem: HasPos;
    fn spacing() -> i32;
    fn as_slice(&self) -> &[Self::Elem];
}

pub trait HasPos: Clone {
    fn pos(&self) -> V2;
    fn pos_mut(&mut self) -> &mut V2;

    fn with_pos(&self, pos: V2) -> Self {
        let mut x = self.clone();
        *x.pos_mut() = pos;
        x
    }
}

impl HasPos for V2 {
    fn pos(&self) -> V2 { *self }
    fn pos_mut(&mut self) -> &mut V2 { self }
    fn with_pos(&self, pos: V2) -> V2 { pos }
}

macro_rules! define_points {
    ($Points:ident : $T:ty; $SPACING:expr) => {
        pub struct $Points {
            pub data: Vec<$T>,
        }

        impl ::cache::Summary for $Points {
            fn alloc() -> Box<$Points> {
                Box::new($Points { data:  Vec::new() })
            }

            fn write_to(&self, mut f: File) -> io::Result<()> {
                use libserver_util::bytes::WriteBytes;
                try!(f.write_bytes(self.data.len() as u32));
                try!(f.write_bytes_slice(&self.data));
                Ok(())
            }

            fn read_from(mut f: File) -> io::Result<Box<$Points>> {
                use std::vec::Vec;
                use libserver_util::bytes::ReadBytes;

                let len = try!(f.read_bytes::<u32>()) as usize;
                let mut result = $Points::alloc();
                result.data = Vec::with_capacity(len);
                unsafe {
                    result.data.set_len(len);
                    try!(f.read_bytes_slice(&mut result.data));
                }
                Ok(result)
            }
        }

        impl ::forest::common::PointsLike for $Points {
            type Elem = $T;

            fn spacing() -> i32 { $SPACING as i32 }

            fn as_slice(&self) -> &[Self::Elem] { &self.data }
        }
    }
}


pub trait GenPass {
    type Key: cache::Key + 'static;
    type Value: cache::Summary + 'static;

    fn field_mut<'a, 'd>(ctx: &'a mut Context<'d>)
                         -> &'a mut Cache<'d, Self::Key, Self::Value>;
    fn generate(ctx: &mut Context,
                key: Self::Key,
                value: &mut Self::Value);
}
