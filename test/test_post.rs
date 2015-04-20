use curl::http;
use super::server;

#[test]
pub fn test_post_binary_with_slice() {
    let srv = server!(
        recv!(b"POST / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                Content-Type: application/octet-stream\r\n\
                Content-Length: 11\r\n\
                \r\n\
                Foo Bar Baz"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n")
    );

    let res = http::handle()
        .post(server::url("/"), "Foo Bar Baz")
        .exec().unwrap();

    srv.assert();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}

#[test]
pub fn test_post_binary_with_string() {
    let srv = server!(
        recv!(b"POST / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                Content-Type: application/octet-stream\r\n\
                Content-Length: 11\r\n\
                \r\n\
                Foo Bar Baz"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n")
    );

    let body = "Foo Bar Baz".to_string();
    let res = http::handle()
        .post(server::url("/"), &body)
        .exec().unwrap();

    srv.assert();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}

#[test]
pub fn test_post_binary_with_reader() {
    let srv = server!(
        recv!(b"POST / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                Transfer-Encoding: chunked\r\n\
                Content-Type: application/octet-stream\r\n\
                \r\n\
                b\r\n\
                Foo Bar Baz"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n")
    );

    let mut body = "Foo Bar Baz".as_bytes();
    let res = http::handle()
        .post(server::url("/"), &mut body)
        .exec().unwrap();

    srv.assert();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}

#[test]
pub fn test_post_binary_with_reader_and_content_length() {
    let srv = server!(
        recv!(b"POST / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                Content-Length: 11\r\n\
                Content-Type: application/octet-stream\r\n\
                \r\n\
                Foo Bar Baz"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n")
    );

    let mut body = "Foo Bar Baz".as_bytes();
    let res = http::handle()
        .post(server::url("/"), &mut body)
        .content_length(11)
        .exec().unwrap();

    srv.assert();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}
