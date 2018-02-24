extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;
extern crate tempdir;

#[macro_use]
extern crate structopt;

use std::io::{self, BufRead};
use std::env;
use std::path::PathBuf;
use tempdir::TempDir;
use structopt::StructOpt;

use manga_rs::{SimpleAdapter, AsciiTester, MemoryStore, Mdbm, process_item, check_item};

const ANAGRAM_DATA_PATH: &str = "ANAGRAM_DATA_PATH";
const MINIMUM_STRING_LEN: usize = 16;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag.
    /// Don't insert into the database
    #[structopt(short = "n", long = "no-write")]
    no_write: bool,

    /// Specify a location to load/store data files
    #[structopt(short = "p", long = "path", parse(from_os_str))]
    path: Option<PathBuf>,

    /// Print more stuff
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}


fn main() {
    let opt = Opt::from_args();

    let stdin = io::stdin();
    let mut adapter = SimpleAdapter::new();
    let mut tester = AsciiTester::default();
    let path = match opt.path {
        Some(p) => p,
        None => TempDir::new("anagrams_rs").unwrap().path().to_owned(),
    };

    let mut store = Mdbm::with_path(&path, 2_000_000);

    for item in stdin.lock().lines() {
        let item = match item {
            Ok(item) => item,
            Err(e) => {
                println!("error in stream: {:?}", e);
                break
            }
        };
        if item.len() < MINIMUM_STRING_LEN { continue }
        if opt.no_write {
            check_item(item, &mut store, &mut adapter, &mut tester);
        } else {
            process_item(item, &mut store, &mut adapter, &mut tester);
        }
    }
    adapter.print_results();
}

