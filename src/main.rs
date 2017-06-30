extern crate gnip_twitter_stream;
extern crate serde_json;

use std::str;
use std::env;
use std::time::Instant;

use gnip_twitter_stream::{load_cred, Tweet, GnipStream};

mod filters;

fn main() {
    let cred_path = match env::var("TWITTER_CRED_PATH") {
        Ok(p) => p,
        Err(e) => panic!("error loading credential {:?}", e),
    };

    let cred = load_cred(&cred_path);
    let url = "https://gnip-stream.twitter.com/stream/sample10/accounts/anagramatron/publishers/twitter/prod.json";

    let mut streamer = GnipStream::new(url, 2);
    streamer.run(&cred).expect("failed to start stream");

    let mut count = 0usize;
    let mut filt_count = 0usize;
    let start = Instant::now();

    while let Some(string) = streamer.next() {
        let string = match string {
            Ok(s) => s,
            Err(e) => {
                println!("ERROR {:?}", e);
                break
            }
        };

        let tweet = match serde_json::from_str::<Tweet>(&string) {
            Ok(t) => t,
            Err(e) => { println!("ERROR {:?}\n{}", e, string); continue },
        };

        count += 1;
        if filters::filter_all(&tweet) {
            filt_count += 1;
            println!("{}", tweet.text);
        }

        if count % 1000 == 0 {
            let elapsed = start.elapsed().as_secs() as usize;
            let tps = count / elapsed;
            let passed = (filt_count as f64 / count as f64) * 100 as f64;
            println!("count: {}/{} ({:.2}%) secs {} ({} tps)", filt_count, count, passed, elapsed, tps);
        }
    }
}
