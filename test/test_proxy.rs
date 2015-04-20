use curl::http;
use super::server;

#[test]
pub fn test_proxy() {
    ::env_logger::init().unwrap();

    let srv = server!(
        recv!(b"POST http://www.google.com/ HTTP/1.1\r\n\
                Host: www.google.com\r\n\
                Accept: */*\r\n\
                Proxy-Connection: Keep-Alive\r\n\
                Content-Type: application/octet-stream\r\n\
                Content-Length: 11\r\n\
                \r\n\
                Foo Bar Baz"),
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n\
                Hello\r\n\r\n"));

    let res = http::handle()
        .proxy(server::url("/"))
        .post("http://www.google.com/", "Foo Bar Baz")
        .exec().unwrap();

    srv.assert();

    assert!(res.get_code() == 200);
    assert!(res.get_body() == "Hello".as_bytes());
}
