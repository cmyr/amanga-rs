use std::io::{self, BufReader, BufRead};
use std::str;
use std::string::FromUtf8Error;
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::time::{Instant, Duration};

use reqwest::header::{Accept, AcceptEncoding, Connection, qitem, Encoding};
use reqwest::{Client, Response, StatusCode, Error as ReqError};
use serde_json::{self, Error as JsonError};

use super::Credential;
use tweet::Tweet;

#[derive(Debug)]
/// Error that occurs when connecting to a url.
pub enum ConnectionError {
    Http(ReqError),
    UnexpectedStatus(StatusCode),
}

#[derive(Debug)]
/// Error that occurs during the course of a connection.
pub enum StreamError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Disconnect,
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

impl From<ReqError> for ConnectionError {
    fn from(error: ReqError) -> ConnectionError {
        ConnectionError::Http(error)
    }
}

impl From<JsonError> for StreamError {
    fn from(error: JsonError) -> StreamError {
        StreamError::Json(error)
    }
}

impl StreamError {
    pub fn is_disconnect(&self) -> bool {
        match *self {
            StreamError::Disconnect | StreamError::Io(_) => true,
            _ => false,
        }
    }
}

/// A single connection to a stream component
pub struct StreamConnection<'a> {
    cred: &'a Credential,
    endpoint: String,
    handle: Option<thread::JoinHandle<()>>,
    send: mpsc::Sender<StreamResult>,
    is_running: Arc<AtomicBool>,
    next_retry: Option<Instant>,
}

pub struct GnipStream<'a> {
    base_url: &'a str,
    part_urls: Vec<String>,
    recv: mpsc::Receiver<StreamResult>,
    send: mpsc::Sender<StreamResult>,
    connections: Vec<StreamConnection<'a>>,
}



impl<'a> StreamConnection<'a> {

    fn connect_stream(&self) -> Result<Response, ConnectionError> {
        eprintln!("connecting to url {}", &self.endpoint);
        let client = Client::new().unwrap();
        let resp = client.get(&self.endpoint)
            .basic_auth(self.cred.user.clone(), Some(self.cred.pw.clone()))
            .header(Accept::json())
            .header(Connection::keep_alive())
            .header(AcceptEncoding(vec![qitem(Encoding::Gzip)]))
            .send()?;

        if resp.status() != &StatusCode::Ok {
            return Err(ConnectionError::UnexpectedStatus(resp.status().to_owned()))
        }
        Ok(resp)
    }

    fn start(&mut self) -> Result<(), ConnectionError> {
        if self.is_running.load(Ordering::Relaxed) {
            eprintln!("connection to url {} already started", &self.endpoint);
            return Ok(())
        }

        let stream = self.connect_stream()?;
        eprintln!("{}, {}", stream.status(), stream.headers());
        self.is_running.store(true, Ordering::Relaxed);

        let chan = self.send.clone();
        let url = self.endpoint.clone();
        let is_running = self.is_running.clone();
        let t = thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            let mut brk = false;
            loop {
                let item = next_in_stream(&mut reader);
                if item.as_ref().err().map(|e| e.is_disconnect()).unwrap_or(false) { brk = true }
                // exit loop after sending errors
                if let Err(e) = chan.send(item) {
                    eprintln!("partition {} channel closed, exiting.\n{:?}", &url, e);
                    return
                }
                if brk {
                    eprintln!("brk set, exiting connection {}", &url);
                    is_running.store(false, Ordering::Relaxed);
                    break
                    }
                }
            });
        self.handle = Some(t);
        Ok(())
    }

    fn retry(&mut self) -> Result<(), ConnectionError> {
        eprintln!("retrying {}", self.endpoint);
        self.handle.take().map(|h| h.join().expect("couldn't join thread"));
        self.start()
    }
}

impl<'a> GnipStream<'a> {
    pub fn new(base_url: &'a str, cred: &'a Credential, parts: usize) -> GnipStream<'a> {
        let (send, recv) = mpsc::channel();
        let part_urls = (1..parts +1)
            .map(|p| format!("{}?partition={}", base_url, p))
            .collect::<Vec<_>>();

        let connections = part_urls.iter().map(|url| {
            StreamConnection {
                cred: cred,
                endpoint: url.to_owned(),
                handle: None,
                send: send.clone(),
                is_running: Arc::new(AtomicBool::new(false)),
                next_retry: None,
            }
        }).collect::<Vec<_>>();

        GnipStream { base_url, part_urls, recv, send, connections }
    }


    pub fn run(&mut self) -> Result<(), ConnectionError> {
        for mut conn in self.connections.iter_mut() {
            conn.start()?
        }
        Ok(())
    }

    fn try_reconnect(&mut self) {
        eprintln!("trying reconnect");
        for mut conn in self.connections.iter_mut() {
            let not_running = !conn.is_running.load(Ordering::Relaxed);
            let should_retry = conn.next_retry.map(|inst| inst >= Instant::now()).unwrap_or(true);
            if not_running && should_retry {
                let next_retry = if conn.retry().is_err() {
                    Some(Instant::now() + Duration::new(30, 0))
                } else {
                    None
                };
                conn.next_retry = next_retry;
            }
        }
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
        let n = self.recv.recv().ok();
        if n.as_ref().map(|n| n.is_err()).unwrap_or(false) {
            self.try_reconnect();
        }
        n
    }
}
