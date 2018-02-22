
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::io;
use std::ffi::OsStr;

use serde::Serialize;
use serde::de::DeserializeOwned;
use chrono::prelude::*;
use gnudbm::{GdbmOpener, RwHandle};

use anagrams::Store;

const ITEMS_PER_FILE: usize = 2_000_000;
const DB_CREATION_DATE_KEY: &str = "net.cmyr.creationDate";

/// Wraps a collection of gdbm files, allowing chunks to be dropped
/// as necessary.
pub struct Mdbm {
    base_path: PathBuf,
    chunk_size: usize,
    chunks: Vec<RwHandle>,
}

impl Mdbm {
    /// Loads or creates a new db collection.
    pub fn with_path<P: AsRef<Path>>(p: P, chunk_size: usize) -> Self {
        let base_path = p.as_ref().to_owned();
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

        Mdbm { base_path, chunk_size, chunks }
    }

    fn add_chunk(&mut self) {
        let now: DateTime<Utc> = Utc::now();
        let filename = now.format("%Y-%m-%d_%H_%M_%S.dbm").to_string();
        let filepath = self.base_path.join(filename);
        let mut db = GdbmOpener::new()
            .create(true)
            .readwrite(&filepath)
            .expect(&format!("failed to create new gdbm section at {:?}", filepath));
        db.store(DB_CREATION_DATE_KEY.as_bytes(), &now).unwrap();
        self.chunks.push(db);
    }

    fn check_health(&mut self) {
        let needs_chunk = match self.chunks.last() {
            Some(last) => {
                // TODO: how efficient is count, exactly?
                let len = last.count().unwrap();
                len >= self.chunk_size
            }
            None => true
        };

        if needs_chunk {
            self.add_chunk();
        }
    }
}

impl<K, V> Store<K, V> for Mdbm
    where K: AsRef<[u8]>,
          V: Serialize + DeserializeOwned,
{

  fn get_item(&self, key: &K) -> Option<V> {
      for chunk in &self.chunks {
          if let Ok(val) = chunk.fetch(key) {
              return val.deserialize().ok()
          }
      }
      None
  }

    fn insert(&mut self, key: K, value: V) {
        self.check_health();
        self.chunks.last_mut().unwrap().store(key, &value).unwrap();
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
            let mut db = Mdbm::with_path(tempdir.path(), 10);
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
        let db = Mdbm::with_path(tempdir.path(), 10);
        assert_eq!(db.chunks.len(), 1);
        let item = db.get_item(&String::from("key 1"));
        assert_eq!(item, Some("value 1".to_string()));
    }
}
