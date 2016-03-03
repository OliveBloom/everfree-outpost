use std::iter;

use libserver_types::*;

use cache::{self, Cache};
use forest2::context::Context;


pub trait GridLike {
    type Elem: Copy;
    fn size() -> V2;
    fn bounds() -> Region<V2> { Region::new(scalar(0), Self::size()) }
    fn get(&self, offset: V2) -> Self::Elem;
    fn set(&mut self, offset: V2, val: Self::Elem);
}

macro_rules! define_grid {
    ($Grid:ident : $T:ty; $SIZE:expr) => {
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

        impl ::forest2::common::GridLike for $Grid {
            type Elem = $T;

            fn size() -> V2 { scalar($SIZE as i32) }

            fn get(&self, offset: V2) -> $T {
                use forest2::common::GridLike;
                self.data[Self::bounds().index(offset)]
            }

            fn set(&mut self, offset: V2, val: $T) {
                use forest2::common::GridLike;
                self.data[Self::bounds().index(offset)] = val;
            }
        }
    };
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
