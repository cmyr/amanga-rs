use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Write;
use std::env;

use serde_json;
use serde::ser::Serialize;
use chrono::{Local, DateTime};
use flate2::Compression;
use flate2::write::GzEncoder;


pub fn write_saved<T: Serialize>(items: &[T], gzip: bool) {
    let save_dir = env::var("TWITTER_SAVE_DIR").expect("expected $TWITTER_SAVE_DIR");
    let now: DateTime<Local> = Local::now();

    let mut path = PathBuf::from(&save_dir);
    path.push(now.format("%Y").to_string());
    path.push(now.format("%m").to_string());
    path.push(now.format("%d").to_string());
    if !&path.exists() {
        fs::create_dir_all(&path).expect(&format!("create_dir failed for {:?}", &path));
    }

    let ext = if gzip { "json.gz" } else { "json" };
    path.push(format!("{}.{}", now.format("%F_%T"), ext));

    let mut output = match File::create(&path) {
        Ok(r) => r,
        Err(e) => panic!("failed to create file at {:?}:\n{:?}", path, e),
    };

    eprintln!("saved file: {:?}", path);
    let to_write: String = serde_json::to_string(items).unwrap();
    let bytes: Vec<u8> = match gzip {
        false => to_write.into_bytes(),
        true => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
            encoder.write_all(to_write.as_bytes()).unwrap();
            encoder.finish().unwrap()
        }
    };
    output.write_all(&bytes).expect("write failed");
}
