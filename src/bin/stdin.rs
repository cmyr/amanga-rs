extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;

use std::io::{self, BufRead};

//use gnip_twitter_stream::Tweet;
use manga_rs::{SimpleAdapter, simple_find_anagrams};

fn main() {

    let mut finder = SimpleAdapter::new();

    let stdin = io::stdin();
    let mut iter = stdin.lock().lines().map(Result::unwrap);
    simple_find_anagrams(&mut iter, &mut finder);
    finder.print_results();
}

