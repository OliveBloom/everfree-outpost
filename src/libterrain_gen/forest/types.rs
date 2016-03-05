struct PlaneGlobals {
    main_seed: u64,
    heightmap_seed: u64,
    inited: bool,
}

const HEIGHTMAP_SIZE: usize = 64;
struct HeightMap {
    buf: [i32; HEIGHTMAP_SIZE * HEIGHTMAP_SIZE],
}

struct CaveDetail {
    buf: Vec<[u8; (CHUNK_SIZE * CHUNK_SIZE) as usize / 8]>,
}

pub struct Context {
    globals: Cache<PlaneGlobals>,
    height_map: Cache<HeightMap>,
    cave_detail: Cache<CaveDetail>,
}

impl Context {
    pub fn globals(&mut self, pid: Stable<PlaneId>) -> &mut PlaneGlobals {
        self.globals.load(pid, scalar(0)).unwrap();
    }
}
