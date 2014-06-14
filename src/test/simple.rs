use {handle};
use super::server;

#[test]
pub fn test_simple_get() {
  let srv = server!(
    recv!(
      "GET / HTTP/1.1\r\n\
       Host: localhost:8482\r\n\
       Accept: */*\r\n\r\n"), // Send the data
    send!(
      "HTTP/1.1 200 OK\r\n\
       Content-Length: 5\r\n\r\n\
       Hello\r\n")); // Sends

  let res = handle()
    .get("http://localhost:8482")
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200, "code is {}", res.get_code());
  assert!(res.get_body() == "Hello".as_bytes());
  assert!(res.get_headers().len() == 1);
  assert!(res.get_header("Content-Length") == ["5".to_string()]);
}

#[test]
pub fn test_get_with_custom_headers() {
  let srv = server!(
    recv!(
      "GET / HTTP/1.1\r\n\
       Host: localhost:8482\r\n\
       Accept: */*\r\n\
       User-Agent: Zomg Test\r\n\r\n"),
    send!(
      "HTTP/1.1 200 OK\r\n\
       Content-Length: 5\r\n\r\n\
       Hello\r\n\r\n"));

  let res = handle()
    .get("http://localhost:8482")
    .header("User-Agent", "Zomg Test")
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200);
  assert!(res.get_body() == "Hello".as_bytes());
  assert!(res.get_headers().len() == 1);
  assert!(res.get_header("Content-Length") == ["5".to_string()]);
}

#[test]
pub fn test_simple_post() {
  let srv = server!(
    recv!(
      "POST / HTTP/1.1\r\n\
       Host: localhost:8482\r\n\
       Accept: */*\r\n\
       Content-Length: 11\r\n\
       Content-Type: application/octet-stream\r\n\
       \r\n\
       Foo Bar Baz"),
    send!(
      "HTTP/1.1 200 OK\r\n\
       Content-Length: 5\r\n\r\n\
       Hello\r\n\r\n"));

  let res = handle()
    .post("http://localhost:8482", "Foo Bar Baz")
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200);
  assert!(res.get_body() == "Hello".as_bytes());
}

#[test]
pub fn test_multiple_get_requests_same_handle() {
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
