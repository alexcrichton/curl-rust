extern crate curl;

use std::io::Read;
use std::str;
use std::time::Duration;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

use curl::easy::{Easy, Form};

use server::Server;
mod server;

fn handle() -> Easy {
    let mut e = Easy::new();
    t!(e.timeout(Duration::new(20, 0)));
    return e
}

#[test]
fn custom() {
    let s = Server::new();
    s.receive("\
POST / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
Content-Length: 142\r\n\
Expect: 100-continue\r\n\
Content-Type: multipart/form-data; boundary=--[..]\r\n\
\r\n\
--[..]\r\n\
Content-Disposition: form-data; name=\"foo\"\r\n\
\r\n\
1234\r\n\
--[..]\r\n");
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mut form = Form::new();
    t!(form.part("foo").contents(b"1234").add());
    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}

#[test]
fn buffer() {
    let s = Server::new();
    s.receive("\
POST / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
Content-Length: 181\r\n\
Expect: 100-continue\r\n\
Content-Type: multipart/form-data; boundary=--[..]\r\n\
\r\n\
--[..]\r\n\
Content-Disposition: form-data; name=\"foo\"; filename=\"bar\"\r\n\
Content-Type: foo/bar\r\n\
\r\n\
1234\r\n\
--[..]\r\n");
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mut form = Form::new();
    t!(form.part("foo")
           .buffer("bar", b"1234".to_vec())
           .content_type("foo/bar")
           .add());
    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}

#[test]
fn file() {
    let s = Server::new();
    let formdata = include_str!("formdata");
    s.receive(format!("\
POST / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
Content-Length: {}\r\n\
Expect: 100-continue\r\n\
Content-Type: multipart/form-data; boundary=--[..]\r\n\
\r\n\
--[..]\r\n\
Content-Disposition: form-data; name=\"foo\"; filename=\"formdata\"\r\n\
Content-Type: application/octet-stream\r\n\
\r\n\
{}\
\r\n\
--[..]\r\n", 199 + formdata.len(), formdata).as_str());
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mut form = Form::new();
    t!(form.part("foo")
           .file("tests/formdata")
           .add());
    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}

#[test]
fn stream() {
    let s = Server::new();
    s.receive("\
POST / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
Content-Length: 238\r\n\
Expect: 100-continue\r\n\
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
--[..]\r\n\
");
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mut form = Form::new();

    let mut contents = "1234".as_bytes();
    t!(form.part("foo").stream(Some(contents.len()), move |buf| {
        Ok(contents.read(buf).unwrap())
    }).add());

    let mut contents = "5678".as_bytes();
    t!(form.part("bar").stream(Some(contents.len()), move |buf| {
        Ok(contents.read(buf).unwrap())
    }).add());

    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}
