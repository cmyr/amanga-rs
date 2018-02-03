#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate reqwest;

mod tweet;
mod stream;
mod error;

pub use stream::GnipStream;
pub use tweet::*;

use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct Credential {
    user: String,
    pw: String,
}

pub fn load_cred(path: &str) -> Credential {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    serde_json::from_slice(&buf).unwrap()
}

