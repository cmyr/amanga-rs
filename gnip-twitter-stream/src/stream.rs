use std::io::{self, BufReader, BufRead};
use std::str;
use std::cmp::{min, max};
use std::string::FromUtf8Error;
use std::thread;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, Duration};

use reqwest::header::{Accept, AcceptEncoding, Connection, qitem, Encoding};
use reqwest::{Client, Response, StatusCode, Error as ReqError};
use serde_json::{self, Error as JsonError};

use super::Credential;
use tweet::Tweet;

static STREAM_TIMEOUT_SECS: u64 = 30;
static STREAM_EMPTY_RETRY_MILLIS: u64 = 100;
static STREAM_MAX_RETRY_SECS: u64 = 300;

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
    Timeout,
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
    //handle: Option<thread::JoinHandle<()>>,
    send: mpsc::Sender<StreamResult>,
    is_running: Arc<AtomicBool>,
    thread_id: usize,
    //next_retry: Option<Instant>,
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
        self.thread_id += 1;

        let chan = self.send.clone();
        let url = self.endpoint.clone();
        let is_running = self.is_running.clone();
        let thread_id = self.thread_id;
        let _ = thread::spawn(move || {
            eprintln!("connection #{} starting thread #{}", &url.chars().last().unwrap(), thread_id);
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
                    break
                    }
                }
            is_running.store(false, Ordering::Relaxed);
            eprintln!("connection #{} exiting thread #{}", &url.chars().last().unwrap(), thread_id);
            });
        Ok(())
    }

    fn retry(&mut self) -> Result<(), ConnectionError> {
        eprintln!("retrying {}", self.endpoint);
        //self.handle.take().map(|h| h.join().expect("couldn't join thread"));
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
                //handle: None,
                send: send.clone(),
                is_running: Arc::new(AtomicBool::new(false)),
                thread_id: 0,
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

    /// get an item, with a timeout
    fn get_next(&self) -> StreamResult {
        let start = Instant::now();
        loop {
            match self.recv.try_recv() {
                Ok(next) => return next,
                Err(mpsc::TryRecvError::Empty) => {
                    if start.elapsed().as_secs() >= STREAM_TIMEOUT_SECS {
                        eprintln!("timeout elapsed");
                        for conn in self.connections.iter() {
                            conn.is_running.store(false, Ordering::Relaxed);
                        }
                        return Err(StreamError::Timeout)
                    } else {
                        thread::sleep(Duration::from_millis(STREAM_EMPTY_RETRY_MILLIS));
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    eprintln!("mpsc disconnected");
                    return Err(StreamError::Disconnect)
                }
            }
        }
    }

    fn try_reconnect(&mut self) {
        eprintln!("trying reconnect");
        for mut conn in self.connections.iter_mut() {
            if conn.is_running.load(Ordering::Relaxed) { continue }
            let mut retry_secs = 1u64;
            loop {
                match conn.retry() {
                    Ok(_) => break,
                    Err(err) => eprintln!("reconnect failed with error: {:?}. Retry in {} seconds",
                                          err, retry_secs),
                }
                thread::sleep(Duration::from_secs(retry_secs));
                retry_secs = min(STREAM_MAX_RETRY_SECS, max(2, retry_secs * retry_secs));
            }
        }
    }
}

fn next_in_stream<R: BufRead>(stream: &mut R) -> StreamResult {
    let mut buf = Vec::new();
    let read_bytes = stream.read_until(b'\r', &mut buf)?;
    if read_bytes == 0 {
        eprintln!("read 0 bytes from stream (disconnect)");
        Err(StreamError::Disconnect)
    } else {
       let msg = String::from_utf8(buf)?;
       Ok(serde_json::from_str(&msg)?)
    }
}

impl<'a> Iterator for GnipStream<'a> {
    type Item = StreamResult;
    fn next(&mut self) -> Option<StreamResult> {
        let next = self.get_next();
        if next.is_err() {
            self.try_reconnect();
        }
        Some(next)
    }
}
