use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::env;

use serde_json;
use serde::ser::Serialize;
use chrono::{Local, DateTime};


pub fn write_saved<T: Serialize>(items: &[T], gzip: bool) {
    let save_dir = env::var("TWITTER_SAVE_DIR").expect("expected $TWITTER_SAVE_DIR");
    let mut path = PathBuf::from(&save_dir);
    let now: DateTime<Local> = Local::now();
    let now_str = format!("{}.json", now.format("%F_%T"));
    path.push(now_str);
    let mut output = File::create(path).expect("failed to create file");
    let to_write: String = serde_json::to_string(items).unwrap();
    let bytes: Vec<u8> = match gzip {
        false => to_write.into_bytes(),
        true => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
            encoder.write(to_write.as_bytes()).unwrap();
            encoder.finish().unwrap()
        };
    }
    output.write_all(&bytes).expect("write failed");
}


