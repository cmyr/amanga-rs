extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;

use std::io::{self, BufRead};

//use gnip_twitter_stream::Tweet;
use manga_rs::AnagramFinder;

fn main() {

    let mut finder = AnagramFinder::new();

    let stdin = io::stdin();
    for string in stdin.lock().lines() {
        let string = string.expect("failed to parse stdin");

        //let tweet = match serde_json::from_str::<Tweet>(&string) {
            //Ok(t) => t,
            //Err(e) => { println!("ERROR {:?}\n{}", e, string); continue },
        //};

        //if filters::filter_all(&tweet) {
            finder.add(&string);
            //finder.add(&tweet.text);
        //}

    }

    finder.print_results();
}

