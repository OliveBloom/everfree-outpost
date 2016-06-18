use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::path::PathBuf;
use linked_hash_map::LinkedHashMap;

use libserver_types::*;
use libserver_config::Storage;


pub trait Key: Hash + Eq + Copy {
    fn to_path(&self) -> PathBuf;
}

impl Key for V2 {
    fn to_path(&self) -> PathBuf {
        PathBuf::from(format!("{},{}", self.x, self.y))
    }
}

impl<ID: Hash + Eq + Copy> Key for Stable<ID> {
    fn to_path(&self) -> PathBuf {
        PathBuf::from(format!("{}", self.unwrap()))
    }
}

impl Key for u8 {
    fn to_path(&self) -> PathBuf {
        PathBuf::from(format!("{}", *self))
    }
}

impl<A: Key, B: Key> Key for (A, B) {
    fn to_path(&self) -> PathBuf {
        self.0.to_path()
            .join(self.1.to_path())
    }
}

impl<A: Key, B: Key, C: Key> Key for (A, B, C) {
    fn to_path(&self) -> PathBuf {
        self.0.to_path()
            .join(self.1.to_path())
            .join(self.2.to_path())
    }
}


pub trait Summary {
    /// Create a new, empty summary.
    fn alloc() -> Box<Self>;

    /// Write the summary data to a file.
    fn write_to(&self, f: File) -> io::Result<()>;

    /// Create a new summary from the contents of a file.
    fn read_from(f: File) -> io::Result<Box<Self>>;
}


struct CacheEntry<T> {
    data: Box<T>,
    dirty: bool,
}

impl<T> CacheEntry<T> {
    fn new(data: Box<T>) -> CacheEntry<T> {
        CacheEntry {
            data: data,
            dirty: false,
        }
    }
}

pub struct Cache<'d, K: Key, T: Summary> {
    storage: &'d Storage,
    name: &'static str,
    cache: LinkedHashMap<K, CacheEntry<T>>,
}

const CACHE_LIMIT: usize = 1024;

impl<'d, K: Key, T: Summary> Cache<'d, K, T> {
    pub fn new(storage: &'d Storage, name: &'static str) -> Cache<'d, K, T> {
        Cache {
            storage: storage,
            name: name,
            cache: LinkedHashMap::new(),
        }
    }

    fn make_space(&mut self, extra: usize) {
        assert!(extra <= CACHE_LIMIT);
        while self.cache.len() + extra > CACHE_LIMIT {
            let (key, entry) = self.cache.pop_front().unwrap();
            if entry.dirty {
                let file = self.storage.create_summary_file(self.name, &key.to_path());
                match entry.data.write_to(file) {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("error writing cache entry to disk: {}",
                              e.description());
                    },
                }
            }
        }
    }

    pub fn create(&mut self, key: K) -> &mut T {
        self.make_space(1);
        self.cache.insert(key, CacheEntry::new(T::alloc()));
        self.get_mut(key)
    }

    pub fn insert(&mut self, key: K, val: Box<T>) -> &mut T {
        self.make_space(1);
        self.cache.insert(key, CacheEntry::new(val));
        self.get_mut(key)
    }

    pub fn load(&mut self, key: K) -> io::Result<()> {
        if let Some(_) = self.cache.get_refresh(&key) {
            // Already in the cache.
            Ok(())
        } else {
            self.make_space(1);
            let file = unwrap!(self.storage.open_summary_file(self.name, &key.to_path()));
            let summary = try!(T::read_from(file));
            self.cache.insert(key, CacheEntry::new(summary));
            Ok(())
        }
    }

    // No explicit `unload` - data is unloaded automatically in LRU fashion.

    pub fn get(&self, key: K) -> &T {
        &self.cache[&key].data
    }

    pub fn get_mut(&mut self, key: K) -> &mut T {
        let entry = &mut self.cache[&key];
        entry.dirty = true;
        &mut entry.data
    }

    pub fn load_or_create(&mut self, key: K) -> &mut T {
        if let Ok(()) = self.load(key) {
            self.get_mut(key)
        } else {
            self.create(key)
        }
    }
}

impl<'d, K: Key, T: Summary> Drop for Cache<'d, K, T> {
    fn drop(&mut self) {
        // Evict everything.
        self.make_space(CACHE_LIMIT);
    }
}
