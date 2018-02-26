
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::io;
use std::ffi::OsStr;
use std::cell::RefCell;
use std::ops::Drop;

use serde::Serialize;
use serde::de::DeserializeOwned;
use chrono::prelude::*;
use gnudbm::{GdbmOpener, RwHandle};
use lru_cache::LruCache;

use anagrams::Store;

const ITEMS_PER_FILE: usize = 2_000_000;
// in # of items
const CACHE_SIZE: usize = 200_000;
const DB_CREATION_DATE_KEY: &str = "net.cmyr.creationDate";

/// Wraps a collection of gdbm files, allowing chunks to be dropped
/// as necessary.
pub struct Mdbm<V: Serialize> {
    base_path: PathBuf,
    cache: RefCell<LruCache<Vec<u8>, V>>,
    chunk_size: usize,
    chunks: Vec<RwHandle>,
    last_chunk_len: usize,
}

impl<V: Serialize> Mdbm<V> {
    /// Loads or creates a new db collection.
    pub fn new<P: AsRef<Path>>(p: P, chunk_size: usize) -> Self {
        let base_path = p.as_ref().to_owned();
        eprintln!("using base path {}", base_path.display());
        if !base_path.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(&base_path)
                .expect("creating mdbm directory failed");
        }

        let mut chunks = BTreeMap::new();
        for fp in iter_dbm_paths(&base_path).expect("failed to load dbm chunks") {
            let mut db = GdbmOpener::new()
                .readwrite(&fp)
                .expect("failed to load db");
            let created_at = {
                let created_at = db.fetch(DB_CREATION_DATE_KEY.as_bytes()).unwrap();
                let created_at: DateTime<Utc> = created_at.deserialize()
                    .expect(&format!("failed to parse created at date for path {:?}", fp));
                created_at
            };
            chunks.insert(created_at, db);
        }

        // this is gross b/c BTreeMap doesn't implement drain; we just used it to sort.
        let keys = chunks.keys().cloned().collect::<Vec<_>>();
        let mut vals = Vec::new();
        for key in keys {
            vals.push(chunks.remove(&key).unwrap());
        }

        let chunks = vals;
        let cache = RefCell::new(LruCache::new(CACHE_SIZE));
        let last_chunk_len = chunks.last().map(|c| c.count().unwrap())
            .unwrap_or(0);

        Mdbm { base_path, cache, chunk_size, chunks, last_chunk_len }
    }

    fn add_chunk(&mut self) {
        let now: DateTime<Utc> = Utc::now();
        let filename = now.format("%Y-%m-%d_%H_%M_%S.dbm").to_string();
        eprintln!("adding chunk {}", filename);
        let filepath = self.base_path.join(filename);
        let mut db = GdbmOpener::new()
            .create(true)
            .readwrite(&filepath)
            .expect(&format!("failed to create new gdbm section at {:?}", filepath));
        db.store(DB_CREATION_DATE_KEY.as_bytes(), &now).unwrap();
        self.chunks.push(db);
        self.last_chunk_len = 0;
    }

    fn check_health(&mut self)
        where V: Serialize,
    {
        let cache_len = self.cache.borrow().len();
        if cache_len > (CACHE_SIZE / 10) * 9 {
            // clear out some cache space
            let mut i = 0;
            for _ in 0..(CACHE_SIZE / 10) {
                let (k, v) = self.cache.borrow_mut().remove_lru().unwrap();
                self.chunks.last_mut().unwrap().store(k, &v).unwrap();
                i += 1;
            }
            // can't ever be called before we add the first chunk
            self.last_chunk_len = self.chunks.last().map(|c| c.count().unwrap())
                .unwrap();
        }
        let needs_chunk = self.chunks.last().is_none() ||
            self.last_chunk_len >= self.chunk_size;

        if needs_chunk {
            self.add_chunk();
        }
    }
}

impl<K, V> Store<K, V> for Mdbm<V>
    where K: AsRef<[u8]>,
          V: Serialize + DeserializeOwned + Clone,
{

  fn get_item(&self, key: &K) -> Option<V> {
      if let Some(val) = self.cache.borrow_mut().get_mut(key.as_ref()) {
          return Some(val.to_owned())
      }

      for chunk in &self.chunks {
          if let Ok(val) = chunk.fetch(key.as_ref()) {
              let val: V = val.deserialize().unwrap();
              self.cache.borrow_mut().insert(key.as_ref().to_owned(), val.clone());
              return Some(val)
          }
      }
      None
  }

    fn insert(&mut self, key: K, value: V) {
        self.check_health();
        self.cache.borrow_mut().insert(key.as_ref().to_owned(), value);
    }
}

impl<V: Serialize> Drop for Mdbm<V> {
    fn drop(&mut self) {
        while let Some((k, v)) = self.cache.borrow_mut().remove_lru() {
            let _ = self.chunks.last_mut().unwrap().store(k, &v);
        }
    }
}

fn iter_dbm_paths(dir: &Path) -> io::Result<Box<Iterator<Item=PathBuf>>> {
    let contents = dir.read_dir()?;
    let iter = contents.flat_map(Result::ok)
        .map(|p| p.path())
        .filter(|p| {
            p.extension().and_then(OsStr::to_str).unwrap_or("") == "dbm"
        });
    Ok(Box::new(iter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn smoke_test() {
        let tempdir = TempDir::new("mdbm_test").unwrap();
        {
            let mut db = Mdbm::new(tempdir.path(), 10);
            assert_eq!(db.chunks.len(), 0);
            for i in 0..5 {
                let key = format!("key {}", i);
                let value = format!("value {}", i);
                db.insert(key, value);
            }
            assert_eq!(db.chunks.len(), 1);
            let item = db.get_item(&String::from("key 1"));
            assert_eq!(item, Some("value 1".to_string()));
        }
        // reopen and check that our data was saved
        let db = Mdbm::new(tempdir.path(), 10);
        assert_eq!(db.chunks.len(), 1);
        let item = db.get_item(&String::from("key 2"));
        assert_eq!(item, Some("value 2".to_string()));
    }
}
