use physics::CHUNK_BITS;
use physics::v3::V3;
use terrain::LOCAL_BITS;


/// A chunk of terrain.  Each element is a block ID.
pub type BlockChunk = [u16; 1 << (3 * CHUNK_BITS)];
/// BlockChunk for every chunk in the local region.
pub type LocalChunks = [BlockChunk; 1 << (2 * LOCAL_BITS)];


bitflags! {
    pub flags TemplateFlags: u8 {
        const HAS_SHADOW =      0x01,
        const HAS_ANIM =        0x02,
        const HAS_LIGHT =       0x04,
    }
}


pub struct StructureTemplate {
    // 0
    pub size: (u8, u8, u8),
    pub _pad1: u8,
    pub shape_idx: u16,
    pub part_idx: u16,
    pub part_count: u8,
    pub vert_count: u8,
    pub layer: u8,
    pub flags: TemplateFlags,

    // 12
    pub light_pos: (u8, u8, u8),
    pub light_color: (u8, u8, u8),
    pub light_radius: u16,

    // 20
}

impl StructureTemplate {
    pub fn size(&self) -> V3 {
        V3::new(self.size.0 as i32,
                self.size.1 as i32,
                self.size.2 as i32)
    }
}

pub struct TemplatePart {
    // 0
    pub vert_idx: u16,
    pub vert_count: u16,
    pub offset: (i16, i16),
    pub sheet: u8,
    pub flags: TemplateFlags,

    // 10
    pub anim_length: i8,
    pub anim_rate: u8,
    pub anim_step: u16,     // x-size of each frame

    // 14
}

pub struct TemplateVertex {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}
