
pub use self::types::*;

pub mod types;
pub mod export;
pub mod import;

pub mod error;
pub mod write;
//pub mod read;

fn padding(len: usize) -> usize {
    (4 - (len % 4)) % 4
}
