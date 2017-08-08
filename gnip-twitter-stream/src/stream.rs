use std::io::{self, BufReader, BufRead};
use std::str;
use std::string::FromUtf8Error;
use std::sync::mpsc;
use std::thread;

use reqwest::header::{Accept, AcceptEncoding, Connection, qitem, Encoding};
use reqwest::{Client, Response, StatusCode, Error as ReqError};
use serde_json::{self, Error as JsonError};

use super::Credential;
use tweet::Tweet;

#[derive(Debug)]
pub enum StreamError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Http(ReqError),
    Disconnect,
    UnexpectedStatus(StatusCode),
    Json(JsonError),
}

pub type StreamResult = Result<Tweet, StreamError>;

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

impl From<ReqError> for StreamError {
    fn from(error: ReqError) -> StreamError {
        StreamError::Http(error)
    }
}

impl From<JsonError> for StreamError {
    fn from(error: JsonError) -> StreamError {
        StreamError::Json(error)
    }
}

pub struct GnipStream<'a> {
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

    fn connect_stream(&self, cred: &Credential, url: &str) -> Result<Response, ReqError> {
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
       let msg = String::from_utf8(buf)?;
       Ok(serde_json::from_str(&msg)?)
    }
}

impl<'a> Iterator for GnipStream<'a> {
    type Item = StreamResult;
    fn next(&mut self) -> Option<StreamResult> {
        self.recv.recv().ok()
    }
}
