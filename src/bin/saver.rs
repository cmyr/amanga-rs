extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;
extern crate chrono;

use std::env;
use std::time::Instant;
use chrono::{Local, DateTime};

use gnip_twitter_stream::{load_cred, GnipStream};
use manga_rs::{filter_all, write_saved};

static SAVE_LENGTH: usize = 25000;

fn main() {
    let cred_path = match env::var("TWITTER_CRED_PATH") {
        Ok(p) => p,
        Err(e) => panic!("error loading credential {:?}", e),
    };

    let _ = env::var("TWITTER_SAVE_DIR").expect("expected $TWITTER_SAVE_DIR");

    let cred = load_cred(&cred_path);
    let url = "https://gnip-stream.twitter.com/stream/sample10/accounts/anagramatron/publishers/twitter/prod.json";
    let mut streamer = GnipStream::new(url, &cred, 2);
    streamer.run().expect("failed to start stream");

    let mut count = 0usize;
    let mut filt_count = 0usize;
    let mut last_print = 0usize;
    let mut last_save = Instant::now();
    let start = Instant::now();

    let mut to_save = Vec::new();

    loop {
        let stream_result = streamer.next().expect("stream connection closed");
        let tweet = match stream_result {
            Ok(t) => t,
            Err(e) => {
                eprintln!("stream returned error\n{:?}", e);
                continue
            }
        };

        count += 1;

        if filter_all(&tweet) {
            filt_count += 1;
            to_save.push(tweet);
        }

        let now: DateTime<Local> = Local::now();
        if to_save.len() == SAVE_LENGTH {
            let elapsed = last_save.elapsed().as_secs();
            let tps = SAVE_LENGTH as u64 / elapsed;
            println!("{}: saving batch, tps {}", now.format("%b %d, %H:%M:%S"), tps);
            write_saved(&to_save, true);
            to_save = Vec::new();
            last_save = Instant::now();
        }

        let elapsed = start.elapsed().as_secs() as usize;
        if filt_count % 1000 == 0 && filt_count != last_print && count > 0 && elapsed > 0 {
            last_print = filt_count;
            let tps = count / elapsed;
            let passed = (filt_count as f64 / count as f64) * 100 as f64;
            println!("count: {}/{} ({:.2}%) secs {} ({} tps)",
            filt_count, count, passed, elapsed, tps);
        }
    }
}
