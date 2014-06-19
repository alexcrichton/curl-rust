use {handle};
use super::server;

#[test]
pub fn test_multiple_requests_same_handle() {
  let srv = server!(
    recv!(
      "GET / HTTP/1.1\r\n\
       Host: localhost:8482\r\n\
       Accept: */*\r\n\r\n"),
    send!(
      "HTTP/1.1 200 OK\r\n\
       Content-Length: 5\r\n\r\n\
       Hello\r\n\r\n"),
    recv!(
      "GET /next HTTP/1.1\r\n\
       Host: localhost:8482\r\n\
       Accept: */*\r\n\r\n"),
    send!(
      "HTTP/1.1 200 OK\r\n\
       Content-Length: 5\r\n\r\n\
       World\r\n\r\n"));

  let mut handle = handle();
  let res1 = handle.get("http://localhost:8482").exec().unwrap();
  let res2 = handle.get("http://localhost:8482/next").exec().unwrap();

  srv.assert();

  assert!(res1.get_code() == 200);
  assert!(res1.get_body() == "Hello".as_bytes());

  assert!(res2.get_code() == 200);
  assert!(res2.get_body() == "World".as_bytes());
}
