use physics::CHUNK_BITS;
use physics::v3::V3;
use terrain::LOCAL_BITS;


/// A chunk of terrain.  Each element is a block ID.
pub type BlockChunk = [u16; 1 << (3 * CHUNK_BITS)];
/// BlockChunk for every chunk in the local region.
pub type LocalChunks = [BlockChunk; 1 << (2 * LOCAL_BITS)];

pub use data::{StructureTemplate, TemplatePart, TemplateVertex};
pub use data::{TemplateFlags, HAS_SHADOW, HAS_ANIM, HAS_LIGHT};
