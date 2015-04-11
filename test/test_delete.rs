use curl::http;
use super::server;

#[test]
pub fn test_delete_with_no_body() {
    let srv = server!(
        recv!(b"DELETE / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                \r\n"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n")
    );

    let res = http::handle()
        .delete(server::url("/"))
        .exec();

    srv.assert();
    let res = res.unwrap();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}

#[test]
pub fn test_delete_binary_with_slice() {
    let srv = server!(
        recv!(b"DELETE / HTTP/1.1\r\n\
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
        .delete(server::url("/"))
        .body("Foo Bar Baz")
        .exec(); // .unwrap();

    srv.assert();
    let res = res.unwrap();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}
