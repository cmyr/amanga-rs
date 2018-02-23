extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;
extern crate serde;

//use std::str;
use std::env;
//use std::time::Instant;

use gnip_twitter_stream::{load_cred, GnipStream};
use manga_rs::{SimpleAdapter, AsciiTester, MemoryStore, process_item, filter_all};

fn main() {
    let cred_path = match env::var("TWITTER_CRED_PATH") {
        Ok(p) => p,
        Err(e) => panic!("error loading credential {:?}", e),
    };

    let cred = load_cred(&cred_path);
    let url = "https://gnip-stream.twitter.com/stream/sample10/accounts/anagramatron/publishers/twitter/prod.json";

    let mut streamer = GnipStream::new(url, &cred, 2);
    streamer.run().expect("failed to start stream");

    //let mut count = 0usize;
    //let mut filt_count = 0usize;
    //let mut last_print = 0usize;
    //let start = Instant::now();
    let mut adapter = SimpleAdapter::new();
    let mut tester = AsciiTester::default();
    let mut store = MemoryStore::new();
    //let mut iter = streamer.flat_map(|item| item.ok());
    //simple_find_anagrams(&mut iter, &mut finder);

    while let Some(stream_result) = streamer.next() {
        let tweet = match stream_result {
            Ok(s) => s,
            Err(e) => { println!("error in stream {:?})", e); return },
        };

        //count += 1;

        if filter_all(&tweet) {
            //filt_count += 1;
            process_item(tweet, &mut store, &mut adapter, &mut tester);
            //finder.add(&tweet);
        }
    }
        //let elapsed = start.elapsed().as_secs() as usize;
        //if filt_count % 100 == 0 && filt_count != last_print && count > 0 && elapsed > 0 {
            //last_print = filt_count;
            //let tps = count / elapsed;
            //let passed = (filt_count as f64 / count as f64) * 100 as f64;
            //println!("count: {}/{} ({:.2}%) secs {} ({} tps)",
            //filt_count, count, passed, elapsed, tps);
        //}
    //}
}
