#![cfg(feature = "mime")]

use curl::{
    easy::{ReadError, SeekResult},
    mime::PartDataHandler,
    Version,
};
use std::{io::SeekFrom, time::Duration};

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

use curl::easy::{Easy, List};

use crate::server::Server;
mod server;

fn handle() -> Easy {
    let mut e = Easy::new();
    t!(e.timeout(Duration::new(20, 0)));
    let mut list = List::new();
    t!(list.append("Expect:"));
    t!(e.http_headers(list));
    e
}

fn multipart_boundary_size() -> usize {
    // Versions before 8.4.0 used a smaller multipart mime boundary, so the
    // exact content-length will differ between versions.
    if Version::get().version_num() >= 0x80400 {
        148
    } else {
        136
    }
}

#[test]
fn data() {
    let s = Server::new();
    s.receive(&format!(
        "\
         POST / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         Content-Length: {}\r\n\
         Content-Type: multipart/form-data; boundary=--[..]\r\n\
         \r\n\
         --[..]\r\n\
         Content-Disposition: form-data; name=\"foo\"; filename=\"data.txt\"\r\n\
         Content-Type: text/plain\r\n\
         Content-Language: en-US\r\n\
         \r\n\
         1234\r\n\
         --[..]\r\n",
        multipart_boundary_size() + 78
    ));
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mime = handle.new_mime();
    let mut part = mime.add_part();
    t!(part.name("foo"));
    t!(part.data(b"1234"));
    t!(part.content_type("text/plain"));
    let mut part_headers = List::new();
    part_headers.append("Content-Language: en-US").unwrap();
    t!(part.headers(part_headers));
    t!(part.filename("data.txt"));
    t!(mime.post());
    t!(handle.url(&s.url("/")));
    t!(handle.perform());
}

#[test]
fn two_parts() {
    let s = Server::new();
    s.receive(&format!(
        "\
         POST / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         Content-Length: {}\r\n\
         Content-Type: multipart/form-data; boundary=--[..]\r\n\
         \r\n\
         --[..]\r\n\
         Content-Disposition: form-data; name=\"foo\"\r\n\
         \r\n\
         1234\r\n\
         --[..]\r\n\
         Content-Disposition: form-data; name=\"bar\"\r\n\
         \r\n\
         5678\r\n\
         --[..]\r\n",
        multipart_boundary_size() + 108
    ));
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mime = handle.new_mime();
    let mut part = mime.add_part();
    t!(part.name("foo"));
    t!(part.data(b"1234"));
    part = mime.add_part();
    t!(part.name("bar"));
    t!(part.data(b"5678"));
    t!(mime.post());
    t!(handle.url(&s.url("/")));
    t!(handle.perform());
}

#[test]
fn handler() {
    let s = Server::new();
    s.receive(&format!(
        "\
         POST / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         Content-Length: {}\r\n\
         Content-Type: multipart/form-data; boundary=--[..]\r\n\
         \r\n\
         --[..]\r\n\
         Content-Disposition: form-data; name=\"foo\"\r\n\
         \r\n\
         1234\r\n\
         --[..]\r\n",
        multipart_boundary_size() + 6
    ));
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mime = handle.new_mime();
    let mut part = mime.add_part();
    t!(part.name("foo"));
    let buf = b"1234".to_vec();
    t!(part.data_handler(buf.len(), ByteVecHandler::new(buf)));
    t!(mime.post());
    t!(handle.url(&s.url("/")));
    t!(handle.perform());
}

#[derive(Debug)]
struct ByteVecHandler {
    data: Vec<u8>,
    pos: usize,
}

impl ByteVecHandler {
    fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }
}

impl PartDataHandler for ByteVecHandler {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        if self.pos > self.data.len() {
            return Ok(0);
        }
        let remaining_data = &mut self.data[self.pos..];
        let len = remaining_data.len().min(data.len());
        data[..len].copy_from_slice(&remaining_data[..len]);
        self.pos += len;
        Ok(len)
    }

    fn seek(&mut self, whence: SeekFrom) -> SeekResult {
        match whence {
            SeekFrom::Start(pos) => {
                if pos > self.data.len() as u64 {
                    return SeekResult::Fail;
                }
                self.pos = pos as usize;
            }
            SeekFrom::End(_) | SeekFrom::Current(_) => return SeekResult::CantSeek,
        }
        SeekResult::Ok
    }
}
