extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;

use std::io::{self, BufRead};

//use gnip_twitter_stream::Tweet;
use manga_rs::{SimpleAdapter, AsciiTester, MemoryStore, process_item};
fn main() {

    let stdin = io::stdin();
    let mut adapter = SimpleAdapter::new();
    let mut tester = AsciiTester;
    let mut store = MemoryStore::new();

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

