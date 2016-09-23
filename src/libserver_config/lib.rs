#![crate_name = "server_config"]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate log;
extern crate rand;

extern crate physics as libphysics;
extern crate server_types as libserver_types;
extern crate rustc_serialize;

pub use data::Data;
pub use storage::Storage;
pub use loot_table::LootTables;

#[macro_use] pub mod data;
pub mod storage;
pub mod loot;
pub mod loot_table;
