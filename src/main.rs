#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

extern crate reqwest;

use std::fs::File;
use std::io::{Read, BufReader, BufRead};
use std::str;
use std::env;
use std::time::Instant;
use std::sync::mpsc;
use std::thread;

use reqwest::header::{Accept, AcceptEncoding, Connection, qitem, Encoding};
use reqwest::Client;

use chrono::{DateTime as ChronoDateTime, Utc};
pub type DateTime = ChronoDateTime<Utc>;

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

fn start_partition_thread(cred: &Credential, url: &str, chan: mpsc::Sender<String>) {
    let client = Client::new().unwrap();
    let stream = client.get(url)
        .basic_auth(cred.user.clone(), Some(cred.pw.clone()))
        .header(Accept::json())
        .header(Connection::keep_alive())
        .header(AcceptEncoding(vec![qitem(Encoding::Gzip)]))
        .send()
        .unwrap();

    println!("{}, {}", stream.status(), stream.headers());

    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        let mut streamer = JsonStreamer::new(&mut reader);
        while let Some(s) = streamer.next() {
            chan.send(s).unwrap();
        }
    });
}

fn main() {
    let cred_path = match env::var("TWITTER_CRED_PATH") {
        Ok(p) => p,
        Err(e) => panic!("error loading credential {:?}", e),
    };

    let cred = load_cred(&cred_path);
    let url = "https://gnip-stream.twitter.com/stream/sample10/accounts/anagramatron/publishers/twitter/prod.json?partition=1";
    let url2 = "https://gnip-stream.twitter.com/stream/sample10/accounts/anagramatron/publishers/twitter/prod.json?partition=2";
    let (send, recv) = mpsc::channel();
    start_partition_thread(&cred, &url, send.clone());
    start_partition_thread(&cred, &url2, send.clone());

    
    let mut count = 0usize;
    let start = Instant::now();

    while let Ok(string) = recv.recv() {
        match serde_json::from_str::<TweetSummary>(&string) {
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
        let _ = self.reader.read_until(b'\r', &mut buf).unwrap();
        String::from_utf8(buf).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetSummary {
    #[serde(rename = "body")]
    pub text: String,
    #[serde(rename = "twitter_lang")]
    pub lang: String,
    pub link: String,
    #[serde(rename = "postedTime")]
    pub posted_time: DateTime,
    #[serde(rename = "actor")]
    pub user: User,
    #[serde(rename = "twitter_entities")]
    pub entities: Entities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entities {
    pub hashtags: Vec<Hashtag>,
    pub urls: Vec<Url>,
    pub user_mentions: Vec<UserMention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub link: String,
    pub display_name: String,
    pub image: String,
    pub preferred_username: String,
    pub verified: bool,
    pub followers_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashtag {
    pub text: String,
    pub indices: (u64, u64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url {
    pub url: String,
    pub expanded_url: String,
    pub indices: (u64, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMention {
    pub screen_name: String,
    pub name: String,
    pub id: u64,
    pub id_str: String,
    pub indices: (u64, u64),
}

#[allow(dead_code)]
mod filters {
    use super::TweetSummary as Tweet;

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
