bitflags! {
    pub flags TerrainChunkFlags: u32 {
        /// Terrain generation for this chunk is still running in a background thread.
        const TC_GENERATION_PENDING = 0x00000001,
    }
}

bitflags! {
    pub flags StructureFlags: u32 {
        const S_HAS_IMPORT_HOOK     = 0x00000001,
        // const S_HAS_EXPORT_HOOK     = 0x00000002, // unimplemented
    }
}
