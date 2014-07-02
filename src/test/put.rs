use http;
use super::server;

#[test]
pub fn test_put_binary_with_slice() {
  let srv = server!(
    recv!(
      b"PUT / HTTP/1.1\r\n\
        Host: localhost:8482\r\n\
        Accept: */*\r\n\
        Content-Type: application/octet-stream\r\n\
        Content-Length: 11\r\n\
        \r\n\
        Foo Bar Baz"),
    send!(
      b"HTTP/1.1 200 OK\r\n\
        Content-Length: 5\r\n\r\n\
        Hello\r\n\r\n"));

  let res = http::handle()
    .put("http://localhost:8482", "Foo Bar Baz")
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200);
  assert!(res.get_body() == "Hello".as_bytes());
}
