extern crate gnip_twitter_stream;
extern crate manga_rs;
extern crate serde_json;
extern crate tempdir;
extern crate anagram_hit_manager as hit_manager;

#[macro_use]
extern crate structopt;

use std::io::{self, BufRead};
use std::env;
use std::path::PathBuf;
use tempdir::TempDir;
use structopt::StructOpt;

use manga_rs::{SimpleAdapter, AsciiTester, MemoryStore, Mdbm, process_item, check_item};
use gnip_twitter_stream::MinimalTweet;
use hit_manager::DbAdapter;

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

    /// Mdbm chunk size
    #[structopt(short = "s", long = "size", default_value = "2000000")]
    mdbm_size: usize,
}

fn main() {
    let opt = Opt::from_args();

    let stdin = io::stdin();
    //let mut adapter = SimpleAdapter::new();
    let mut adapter = DbAdapter::new();
    let mut tester = AsciiTester::default();
    let path = match opt.path {
        Some(p) => p,
        None => TempDir::new("anagrams_rs").unwrap().path().to_owned(),
    };

    //let mut store = Mdbm::new(&path, opt.mdbm_size);
    let mut store = MemoryStore::new();

    for item in stdin.lock().lines() {
        let raw_item = item.expect("erorr in stream");
        let item = match serde_json::from_str::<MinimalTweet>(&raw_item) {
            Ok(item) => item,
            Err(e) => {
                println!("error decoding item {:?}", e);
                break
            }
        };
        if item.text.len() < MINIMUM_STRING_LEN { continue }
        if opt.no_write {
            check_item(item, &mut store, &mut adapter, &mut tester);
        } else {
            process_item(item, &mut store, &mut adapter, &mut tester);
        }
    }

    eprintln!("found {} hits", adapter.count());
    if opt.verbose {
        let mut hits = adapter.get_hits(None, 50, None);
        while hits.len() > 0 {
            for hit in hits.iter() {
                //let p1: MinimalTweet = serde_json::from_str(&hit.one).unwrap();
                //let p2: MinimalTweet = serde_json::from_str(&hit.two).unwrap();
                eprintln!("======\n{} \t\t// {}\n------\n{} \t\t// {}", hit.one.text, hit.one.link(), hit.two.text, hit.two.link());
            }
            let max_id = hits.iter().map(|h| h.hit.id).max();
            hits = adapter.get_hits(None, 50, max_id);
        }
    }

    //adapter.print_results();
}

