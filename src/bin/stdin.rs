extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;
extern crate tempdir;

use std::io::{self, BufRead};
use tempdir::TempDir;

use manga_rs::{SimpleAdapter, AsciiTester, MemoryStore, Mdbm, process_item};
fn main() {

    let stdin = io::stdin();
    let mut adapter = SimpleAdapter::new();
    let mut tester = AsciiTester;

    let path = TempDir::new("anagrams_rs").unwrap();
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

