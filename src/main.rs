#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

extern crate reqwest;

mod tweet;

use std::fs::File;
use std::io::{self, Read, BufReader, BufRead};
use std::str;
use std::string::FromUtf8Error;
use std::env;
use std::time::Instant;
use std::sync::mpsc;
use std::thread;

use reqwest::header::{Accept, AcceptEncoding, Connection, qitem, Encoding};
use reqwest::{Client, Response, StatusCode, Error as reqError};

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

#[derive(Debug)]
pub enum StreamError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Http(reqError),
    Disconnect,
    UnexpectedStatus(StatusCode),

}

pub type StreamResult = Result<String, StreamError>;

impl From<io::Error> for StreamError {
    fn from(error: io::Error) -> StreamError {
        StreamError::Io(error)
    }
}

impl From<FromUtf8Error> for StreamError {
    fn from(error: FromUtf8Error) -> StreamError {
        StreamError::Utf8(error)
    }
}

impl From<reqError> for StreamError {
    fn from(error: reqError) -> StreamError {
        StreamError::Http(error)
    }
}


struct GnipStream<'a> {
    base_url: &'a str,
    parts: usize,
    running: bool,
    recv: mpsc::Receiver<StreamResult>,
    send: mpsc::Sender<StreamResult>,
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

    fn connect_stream(&self, cred: &Credential, url: &str) -> Result<Response, reqError> {
        eprintln!("connecting to url {}", url);
        let client = Client::new().unwrap();
        client.get(url)
            .basic_auth(cred.user.clone(), Some(cred.pw.clone()))
            .header(Accept::json())
            .header(Connection::keep_alive())
            .header(AcceptEncoding(vec![qitem(Encoding::Gzip)]))
            .send()
    }

    pub fn run(&mut self, cred: &Credential) -> Result<(), StreamError> {
        if self.running {
            eprintln!("stream is already running");
            return Ok(())
        }

        for part in 1..self.parts + 1 {
            let url = format!("{}?partition={}", self.base_url, part);
            let stream = self.connect_stream(cred, &url)?;

            eprintln!("{}, {}", stream.status(), stream.headers());
            if stream.status() != &StatusCode::Ok {
                return Err(StreamError::UnexpectedStatus(stream.status().to_owned()))
            }

            let chan = self.send.clone();
            let t = thread::spawn(move || {
                let mut reader = BufReader::new(stream);
                let mut brk = false;
                loop {
                    let item = next_in_stream(&mut reader);
                    // exit loop after sending errors
                    if item.is_err() { brk = true }
                    if let Err(e) = chan.send(item) {
                        eprintln!("partition {} channel closed, exiting.\n{:?}", part, e);
                        return
                    }
                    if brk { break }
                }
                //TODO: handle this exit
            });
            self.handles.push(t);
        }
        self.running = true;
        Ok(())
    }
}

fn next_in_stream<R: BufRead>(stream: &mut R) -> StreamResult {
    let mut buf = Vec::new();
    let read_bytes = stream.read_until(b'\r', &mut buf)?;
    if read_bytes == 0 {
        Err(StreamError::Disconnect)
    } else {
        Ok(String::from_utf8(buf)?)
    }
}

impl<'a> Iterator for GnipStream<'a> {
    type Item = StreamResult;
    fn next(&mut self) -> Option<StreamResult> {
        self.recv.recv().ok()
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

#[allow(dead_code)]
mod filters {
    use super::Tweet;

    type Filter = fn(&Tweet) -> bool;

    pub fn url_filter(tweet: &Tweet) -> bool {
        tweet.entities.urls.is_empty()
    }

    pub fn manual_url_filter(tweet: &Tweet) -> bool {
        tweet.text.find("https://t.co").is_none()
    }

    pub fn mention_filter(tweet: &Tweet) -> bool {
        tweet.entities.user_mentions.is_empty()
    }

    pub fn en_filter(tweet: &Tweet) -> bool {
        tweet.lang == "en"
    }

    /// Whether or not some percentage of characters are letters.
    pub fn letterish(tweet: &Tweet) -> bool {
        let mut total_chars = 0;
        let mut letter_chars = 0;
        for chr in tweet.text.chars() {
            total_chars += 1;
            // ascii letters + space
            match u32::from(chr) {
                32 | 65 ... 91 | 97 ... 123 => letter_chars += 1,
                _ => (),
            }
        }
        letter_chars as f64 / total_chars as f64 >= 0.7
    }

    pub fn filter_all(tweet: &Tweet) -> bool {
        mention_filter(tweet) &&
        url_filter(tweet) &&
        en_filter(tweet) &&
        manual_url_filter(tweet) &&
        letterish(tweet)
    }
}
