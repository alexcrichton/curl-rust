use curl::http::handle;
use super::server;

#[test]
pub fn test_get_requests() {
    let srv = server!(
        recv!(b"GET / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\r\n"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n"),
        recv!(b"GET /next HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\r\n"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                World\r\n\r\n")
    );

    let mut handle = handle();
    let res1 = handle.get(server::url("/")).exec().unwrap();
    let res2 = handle.get(server::url("/next")).exec().unwrap();

    srv.assert();

    assert!(res1.get_code() == 200);
    assert!(res1.get_body() == "Hello".as_bytes());

    assert!(res2.get_code() == 200);
    assert!(res2.get_body() == "World".as_bytes());
}

#[test]
pub fn test_post_get_requests() {
    let srv = server!(
        recv!(b"POST / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                Content-Type: application/octet-stream\r\n\
                Content-Length: 5\r\n\
                \r\n\
                Hello"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                World\r\n\r\n"),
        recv!(b"GET /next HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\r\n"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 4\r\n\r\n\
                NEXT\r\n\r\n")
    );

    let mut handle = handle().timeout(1000);
    let res1 = handle.post(server::url("/"), "Hello").exec().unwrap();
    let res2 = handle.get(server::url("/next")).exec().unwrap();

    srv.assert();

    assert!(res1.get_code() == 200);
    assert!(res1.get_body() == "World".as_bytes(), "actual={}",
            String::from_utf8_lossy(res1.get_body()));

    assert!(res2.get_code() == 200);
    assert!(res2.get_body() == "NEXT".as_bytes(), "actual={}",
            String::from_utf8_lossy(res2.get_body()));
}
