use curl::http::handle;
use super::server;

#[test]
pub fn test_simple_get() {
  let srv = server!(
    recv!(
      b"GET / HTTP/1.1\r\n\
        Host: localhost:{PORT}\r\n\
        Accept: */*\r\n\r\n"), // Send the data
    send!(
      b"HTTP/1.1 200 OK\r\n\
        Content-Length: 5\r\n\r\n\
        Hello\r\n")); // Sends

  let res = handle()
    .get(server::url("/"))
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200, "code is {}", res.get_code());
  assert!(res.get_body() == "Hello".as_bytes());
  assert!(res.get_headers().len() == 1);
  assert!(res.get_header("content-length") == ["5".to_string()]);
}

#[test]
pub fn test_get_with_custom_headers() {
  let srv = server!(
    recv!(
      b"GET / HTTP/1.1\r\n\
        Host: localhost:{PORT}\r\n\
        Accept: */*\r\n\
        User-Agent: Zomg Test\r\n\r\n"),
    send!(
      b"HTTP/1.1 200 OK\r\n\
        Content-Length: 5\r\n\r\n\
        Hello\r\n\r\n"));

  let res = handle()
    .get(server::url("/"))
    .header("User-Agent", "Zomg Test")
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200);
  assert!(res.get_body() == "Hello".as_bytes());
  assert!(res.get_headers().len() == 1);
  assert!(res.get_header("content-length") == ["5".to_string()]);
}

#[test]
pub fn test_get_tracking_progress() {
  let srv = server!(
    recv!(
      b"GET / HTTP/1.1\r\n\
        Host: localhost:{PORT}\r\n\
        Accept: */*\r\n\r\n"),
    send!(
      b"HTTP/1.1 200 OK\r\n\
        Content-Length: 5\r\n\r\n\
        Hello\r\n"));

  let mut dl = 0;
  let mut cnt = 0;

  let res = handle()
    .get(server::url("/"))
    .progress(|_, dlnow, _, _| {
      cnt = cnt + 1;
      dl = dlnow
    })
    .exec().unwrap();

  srv.assert();

  assert!(res.get_code() == 200);
  assert!(dl == 5);
  assert!(cnt > 1);
}

#[test]
pub fn follows_redirects() {
  let srv1 = server!(
    recv!(
      b"GET / HTTP/1.1\r\n\
        Host: localhost:{PORT}\r\n\
        Accept: */*\r\n\r\n"),
    send!(
      b"HTTP/1.1 301 Moved Permanently\r\n\
        Location: http://localhost:{PORT}/test\r\n\r\n"));
  let srv2 = server!(
    recv!(
      b"GET /test HTTP/1.1\r\n\
        Host: localhost:{PORT}\r\n\
        Accept: */*\r\n\r\n"),
    send!(
      b"HTTP/1.1 200 OK\r\n\r\n\
      response!"));

  let res = handle()
    .get(server::url("/"))
    .follow_redirects(true)
    .exec()
    .unwrap();

  srv1.assert();
  srv2.assert();

  assert!(res.get_code() == 200);
  assert_eq!(res.get_body(), b"response!");
}
