#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

extern crate reqwest;

mod tweet;

use std::fs::File;
use std::io::{Read, BufReader, BufRead};
use std::str;
use std::env;
use std::time::Instant;
use std::sync::mpsc;
use std::thread;

use reqwest::header::{Accept, AcceptEncoding, Connection, qitem, Encoding};
use reqwest::Client;

use tweet::Tweet;

#[derive(Deserialize)]
struct Credential {
    user: String,
    pw: String,
}

fn load_cred(path: &str) -> Credential {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    serde_json::from_slice(&buf).unwrap()
}

struct GnipStream<'a> {
    base_url: &'a str,
    parts: usize,
    running: bool,
    pub recv: mpsc::Receiver<String>,
    send: mpsc::Sender<String>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl<'a> GnipStream<'a> {
    pub fn new(base_url: &'a str, parts: usize) -> GnipStream<'a> {
        let (send, recv) = mpsc::channel();
        GnipStream {
            base_url: base_url,
            parts: parts,
            running: false,
            recv: recv,
            send: send,
            handles: Vec::new(),
        }
    }

    pub fn run(&mut self, cred: &Credential) {
        if self.running {
            eprintln!("stream is already running");
            return
        }

        for part in 1..self.parts + 1 {
            let url = format!("{}?partition={}", self.base_url, part);
            eprintln!("connecting to url {}", url);

            let client = Client::new().unwrap();
            let stream = client.get(&url)
                .basic_auth(cred.user.clone(), Some(cred.pw.clone()))
                .header(Accept::json())
                .header(Connection::keep_alive())
                .header(AcceptEncoding(vec![qitem(Encoding::Gzip)]))
                .send()
                .unwrap();

            eprintln!("{}, {}", stream.status(), stream.headers());

            let chan = self.send.clone();
            let t = thread::spawn(move || {
                let mut reader = BufReader::new(stream);
                let mut streamer = JsonStreamer::new(&mut reader);
                while let Some(s) = streamer.next() {
                    match chan.send(s) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("partition {} channel closed: {:?}", part, e);
                            break
                        }
                    }
                }
                //TODO: handle this exit
            });
            self.handles.push(t);
        }
        self.running = true;
    }
}

fn main() {
    let cred_path = match env::var("TWITTER_CRED_PATH") {
        Ok(p) => p,
        Err(e) => panic!("error loading credential {:?}", e),
    };

    let cred = load_cred(&cred_path);
    let url = "https://gnip-stream.twitter.com/stream/sample10/accounts/anagramatron/publishers/twitter/prod.json";

    let mut streamer = GnipStream::new(url, 2);
    streamer.run(&cred);

    let mut count = 0usize;
    let start = Instant::now();

    while let Ok(string) = streamer.recv.recv() {
        match serde_json::from_str::<Tweet>(&string) {
            Ok(t) => println!("{}: {}", t.lang, t.text),
            Err(e) => println!("ERROR {:?}\n{}", e, string),
        }
        count += 1;
        if count % 1000 == 0 {
            let elapsed = start.elapsed().as_secs();
            let tps = count as i64 / elapsed as i64;
            println!("count: {}, secs {} ({} tps)", count, elapsed, tps);
        }
    }
}

struct JsonStreamer<'a, R> where R: BufRead + 'a {
    reader: &'a mut R,
}

impl <'a, R> JsonStreamer<'a, R> where R: BufRead + 'a {
    pub fn new(reader: &'a mut R) -> Self {
        JsonStreamer {
            reader: reader,
        }
    }
}

impl <'a, R>Iterator for JsonStreamer<'a, R> where R: BufRead + 'a {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut buf = Vec::new();
        //TODO: handle these two errors
        let _ = self.reader.read_until(b'\r', &mut buf).unwrap();
        String::from_utf8(buf).ok()
    }
}
#[allow(dead_code)]
mod filters {
    use super::Tweet;

    type Filter = fn(&Tweet) -> bool;

    pub fn url_filter(tweet: &Tweet) -> bool {
        tweet.entities.urls.is_empty()
    }

    pub fn mention_filter(tweet: &Tweet) -> bool {
        tweet.entities.user_mentions.is_empty()
    }

    pub fn filter_all(tweet: &Tweet) -> bool {
        url_filter(tweet) &&
        mention_filter(tweet)
    }
}
