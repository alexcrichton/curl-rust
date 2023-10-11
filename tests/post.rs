use curl::Version;
use std::time::Duration;

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

use curl::easy::{Easy, Form, List};

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
fn custom() {
    multipart_boundary_size();
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
    let mut form = Form::new();
    t!(form.part("foo").contents(b"1234").add());
    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}

#[cfg(feature = "static-curl")]
#[test]
fn buffer() {
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
         Content-Disposition: form-data; name=\"foo\"; filename=\"bar\"\r\n\
         Content-Type: foo/bar\r\n\
         \r\n\
         1234\r\n\
         --[..]\r\n",
        multipart_boundary_size() + 45
    ));
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mut form = Form::new();
    t!(form
        .part("foo")
        .buffer("bar", b"1234".to_vec())
        .content_type("foo/bar")
        .add());
    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}

#[cfg(feature = "static-curl")]
#[test]
fn file() {
    let s = Server::new();
    let formdata = include_str!("formdata");
    s.receive(
        format!(
            "\
             POST / HTTP/1.1\r\n\
             Host: 127.0.0.1:$PORT\r\n\
             Accept: */*\r\n\
             Content-Length: {}\r\n\
             Content-Type: multipart/form-data; boundary=--[..]\r\n\
             \r\n\
             --[..]\r\n\
             Content-Disposition: form-data; name=\"foo\"; filename=\"formdata\"\r\n\
             Content-Type: application/octet-stream\r\n\
             \r\n\
             {}\
             \r\n\
             --[..]\r\n",
            multipart_boundary_size() + 63 + formdata.len(),
            formdata
        )
        .as_str(),
    );
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut handle = handle();
    let mut form = Form::new();
    t!(form.part("foo").file("tests/formdata").add());
    t!(handle.url(&s.url("/")));
    t!(handle.httppost(form));
    t!(handle.perform());
}
