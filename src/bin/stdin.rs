extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;
extern crate tempdir;

use std::io::{self, BufRead};
use std::env;
use std::path::PathBuf;
use tempdir::TempDir;

use manga_rs::{SimpleAdapter, AsciiTester, MemoryStore, Mdbm, process_item};

const ANAGRAM_DATA_PATH: &str = "ANAGRAM_DATA_PATH";

fn main() {

    let stdin = io::stdin();
    let mut adapter = SimpleAdapter::new();
    let mut tester = AsciiTester::default();
    let path = match env::var(ANAGRAM_DATA_PATH) {
        Ok(p) => PathBuf::from(p),
        Err(_) => TempDir::new("anagrams_rs").unwrap().path().to_owned(),
    };

    let mut store = Mdbm::with_path(&path, 2_000_000);

    for item in stdin.lock().lines() {
        let item = match item {
            Ok(item) => item,
            Err(e) => {
                println!("error in stream: {:?}", e);
                break
            }
        };
        process_item(item, &mut store, &mut adapter, &mut tester);
    }
    adapter.print_results();
}

