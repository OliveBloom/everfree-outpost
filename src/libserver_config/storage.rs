use std::fmt::Debug;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use libserver_types::{Stable, PlaneId, TerrainChunkId};


const DATA_DIR: &'static str = "data";
const BLOCK_DATA_FILE: &'static str = "blocks.json";
const ITEM_DATA_FILE: &'static str = "items.json";
const RECIPE_DATA_FILE: &'static str = "recipes.json";
const TEMPLATE_DATA_FILE: &'static str = "structures.json";
const ANIMATION_DATA_FILE: &'static str = "animations.json";
const SPRITE_LAYER_DATA_FILE: &'static str = "sprite_layers.json";
const LOOT_TABLE_DATA_FILE: &'static str = "loot_tables.json";

const SCRIPT_DIR: &'static str = "scripts";

const SAVE_DIR: &'static str = "save";
const CLIENT_DIR: &'static str = "clients";
const PLANE_DIR: &'static str = "planes";
const SUMMARY_DIR: &'static str = "summary";
const TERRAIN_CHUNK_DIR: &'static str = "terrain_chunks";
const WORLD_FILE_NAME: &'static str = "world.dat";
const RESTART_FILE_NAME: &'static str = "restart.dat";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SaveLayer {
    Base,
    Commit,
    Tmp,
    Delta,
}

impl SaveLayer {
    pub fn dir_name(&self) -> &'static str {
        match *self {
            SaveLayer::Base => "base",
            SaveLayer::Commit => "commit",
            SaveLayer::Tmp => "tmp",
            SaveLayer::Delta => "delta",
        }
    }
}


pub struct Storage {
    base: PathBuf,

    // TODO: using atomicbool here is pretty bad, it doesn't provide any real safety.
    // With current usage, it's okay because two threads never modify the same file at the same
    // time.  In the long run it's probably better to split Storage into separate pieces that
    // properly reflect this separation.
    has_save_base: AtomicBool,
    has_save_commit: AtomicBool,
    has_save_tmp: AtomicBool,
    has_save_delta: AtomicBool,
}

fn init_save_subdir(base: &Path, layer: SaveLayer) -> bool {
    let path = base.join(SAVE_DIR).join(layer.dir_name());
    let exists = path.exists();
    if exists {
        fs::create_dir_all(path.join(CLIENT_DIR)).unwrap();
        fs::create_dir_all(path.join(PLANE_DIR)).unwrap();
        fs::create_dir_all(path.join(TERRAIN_CHUNK_DIR)).unwrap();
    }
    exists
}

impl Storage {
    pub fn new<P: AsRef<Path>>(base: &P) -> Storage {
        let base = base.as_ref().to_owned();

        let has_save_base = init_save_subdir(&base, SaveLayer::Base);
        let has_save_commit = init_save_subdir(&base, SaveLayer::Commit);
        let has_save_tmp = init_save_subdir(&base, SaveLayer::Tmp);
        let has_save_delta = init_save_subdir(&base, SaveLayer::Delta);

        Storage {
            base: base,

            has_save_base: AtomicBool::new(has_save_base),
            has_save_commit: AtomicBool::new(has_save_commit),
            has_save_tmp: AtomicBool::new(has_save_tmp),
            has_save_delta: AtomicBool::new(has_save_delta),
        }
    }

    pub fn create_save_layer(&self, layer: SaveLayer) {
        assert!(!self.has_save_layer(layer));
        fs::create_dir_all(self.base.join(SAVE_DIR).join(layer.dir_name())).unwrap();
        let ok = init_save_subdir(&self.base, layer);
        assert!(ok);
        self.set_has_save_layer(layer, true);

    }

    pub fn has_save_layer(&self, layer: SaveLayer) -> bool {
        match layer {
            SaveLayer::Base => self.has_save_base.load(Ordering::SeqCst),
            SaveLayer::Commit => self.has_save_commit.load(Ordering::SeqCst),
            SaveLayer::Tmp => self.has_save_tmp.load(Ordering::SeqCst),
            SaveLayer::Delta => self.has_save_delta.load(Ordering::SeqCst),
        }
    }

    fn set_has_save_layer(&self, layer: SaveLayer, value: bool) {
        match layer {
            SaveLayer::Base => self.has_save_base.store(value, Ordering::SeqCst),
            SaveLayer::Commit => self.has_save_commit.store(value, Ordering::SeqCst),
            SaveLayer::Tmp => self.has_save_tmp.store(value, Ordering::SeqCst),
            SaveLayer::Delta => self.has_save_delta.store(value, Ordering::SeqCst),
        }
    }

    pub fn remove_save_layer(&self, layer: SaveLayer) {
        assert!(self.has_save_layer(layer));
        fs::remove_dir_all(self.base.join(SAVE_DIR).join(layer.dir_name())).unwrap();
        self.set_has_save_layer(layer, false);
    }

    pub fn move_save_layer(&self, layer1: SaveLayer, layer2: SaveLayer) {
        assert!(self.has_save_layer(layer1));
        assert!(!self.has_save_layer(layer2));
        fs::rename(
            self.base.join(SAVE_DIR).join(layer1.dir_name()),
            self.base.join(SAVE_DIR).join(layer2.dir_name())).unwrap();
        self.set_has_save_layer(layer1, false);
        self.set_has_save_layer(layer2, true);
    }

    pub fn move_save_layer_contents(&self, layer1: SaveLayer, layer2: SaveLayer) {
        assert!(self.has_save_layer(layer1));
        assert!(self.has_save_layer(layer2));

        move_dir_contents(
            &self.base.join(SAVE_DIR).join(layer1.dir_name()),
            &self.base.join(SAVE_DIR).join(layer2.dir_name()));
    }


    // Data files (read-only)

    fn data_path(&self, file: &str) -> PathBuf {
        self.base.join(DATA_DIR).join(file)
    }

    pub fn open_block_data(&self) -> File {
        File::open(self.data_path(BLOCK_DATA_FILE)).unwrap()
    }

    pub fn open_item_data(&self) -> File {
        File::open(self.data_path(ITEM_DATA_FILE)).unwrap()
    }

    pub fn open_recipe_data(&self) -> File {
        File::open(self.data_path(RECIPE_DATA_FILE)).unwrap()
    }

    pub fn open_template_data(&self) -> File {
        File::open(self.data_path(TEMPLATE_DATA_FILE)).unwrap()
    }

    pub fn open_animation_data(&self) -> File {
        File::open(self.data_path(ANIMATION_DATA_FILE)).unwrap()
    }

    pub fn open_sprite_layer_data(&self) -> File {
        File::open(self.data_path(SPRITE_LAYER_DATA_FILE)).unwrap()
    }

    pub fn open_loot_table_data(&self) -> File {
        File::open(self.data_path(LOOT_TABLE_DATA_FILE)).unwrap()
    }


    // Misc

    pub fn script_dir(&self) -> PathBuf {
        self.base.join(SCRIPT_DIR)
    }


    fn restart_file_path(&self) -> PathBuf {
        self.base.join(SAVE_DIR).join(RESTART_FILE_NAME)
    }

    pub fn open_restart_file(&self) -> Option<File> {
        try_open_file(self.restart_file_path())
    }

    pub fn create_restart_file(&self) -> File {
        File::create(self.restart_file_path()).unwrap()
    }

    pub fn remove_restart_file(&self) {
        fs::remove_file(self.restart_file_path()).unwrap()
    }


    // Save files (read-write, multiple layers)

    fn open_save_file_in_layer(&self, rel_path: &Path, layer: SaveLayer) -> Option<File> {
        if !self.has_save_layer(layer) {
            return None;
        }
        try_open_file(self.base.join(SAVE_DIR).join(layer.dir_name()).join(rel_path))
    }

    fn open_save_file(&self, rel_path: &Path) -> Option<File> {
        if let Some(f) = self.open_save_file_in_layer(rel_path, SaveLayer::Base) {
            Some(f)
        } else if let Some(f) = self.open_save_file_in_layer(rel_path, SaveLayer::Commit) {
            Some(f)
        } else if let Some(f) = self.open_save_file_in_layer(rel_path, SaveLayer::Tmp) {
            Some(f)
        } else if let Some(f) = self.open_save_file_in_layer(rel_path, SaveLayer::Delta) {
            Some(f)
        } else {
            None
        }
    }

    fn create_save_file(&self, rel_path: &Path) -> File {
        File::create(self.base.join(SAVE_DIR)
                     .join(SaveLayer::Delta.dir_name())
                     .join(rel_path)).unwrap()
    }


    fn world_rel_path(&self) -> PathBuf {
        PathBuf::new().join(WORLD_FILE_NAME)
    }

    fn client_rel_path(&self, uid: u32) -> PathBuf {
        PathBuf::new().join(CLIENT_DIR)
            .join(format!("{:x}", uid))
            .with_extension("client")
    }

    fn plane_rel_path(&self, stable_pid: Stable<PlaneId>) -> PathBuf {
        PathBuf::new().join(PLANE_DIR)
            .join(format!("{:x}", stable_pid.unwrap()))
            .with_extension("plane")
    }

    fn terrain_chunk_rel_path(&self, stable_tcid: Stable<TerrainChunkId>) -> PathBuf {
        PathBuf::new().join(TERRAIN_CHUNK_DIR)
            .join(format!("{:x}", stable_tcid.unwrap()))
            .with_extension("terrain_chunk")
    }


    pub fn open_world_file(&self) -> Option<File> {
        self.open_save_file(&self.world_rel_path())
    }

    pub fn open_client_file(&self, uid: u32) -> Option<File> {
        self.open_save_file(&self.client_rel_path(uid))
    }

    pub fn open_plane_file(&self, stable_pid: Stable<PlaneId>) -> Option<File> {
        self.open_save_file(&self.plane_rel_path(stable_pid))
    }

    pub fn open_terrain_chunk_file(&self, stable_tcid: Stable<TerrainChunkId>) -> Option<File> {
        self.open_save_file(&self.terrain_chunk_rel_path(stable_tcid))
    }


    pub fn create_world_file(&self) -> File {
        self.create_save_file(&self.world_rel_path())
    }

    pub fn create_client_file(&self, uid: u32) -> File {
        self.create_save_file(&self.client_rel_path(uid))
    }

    pub fn create_plane_file(&self, stable_pid: Stable<PlaneId>) -> File {
        self.create_save_file(&self.plane_rel_path(stable_pid))
    }

    pub fn create_terrain_chunk_file(&self, stable_tcid: Stable<TerrainChunkId>) -> File {
        self.create_save_file(&self.terrain_chunk_rel_path(stable_tcid))
    }


    // Summary

    fn summary_file_path(&self,
                         name: &str,
                         suffix: &Path) -> PathBuf {
        self.base.join(SAVE_DIR).join(SUMMARY_DIR)
            .join(name)
            .join(suffix)
    }

    pub fn open_summary_file(&self,
                             name: &str,
                             suffix: &Path) -> Option<File> {
        try_open_file(self.summary_file_path(name, suffix))
    }

    pub fn create_summary_file(&self,
                               name: &str,
                               suffix: &Path) -> File {
        let path = self.summary_file_path(name, suffix);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        File::create(path).unwrap()
    }
}

fn try_open_file<P: AsRef<Path>+Debug>(path: P) -> Option<File> {
    match File::open(path) {
        Ok(f) => Some(f),
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => None,
                _ => panic!("error opening file: {}", e),
            }
        },
    }
}

fn move_dir_contents(dir1: &Path, dir2: &Path) {
    fs::create_dir_all(dir2).unwrap();
    for entry in fs::read_dir(dir1).unwrap() {
        let entry = entry.unwrap();

        let path1 = dir1.join(entry.file_name());
        let path2 = dir2.join(entry.file_name());

        if entry.metadata().unwrap().is_dir() {
            move_dir_contents(&path1, &path2);
        } else {
            fs::rename(&path1, &path2).unwrap();
        }
    }
}
