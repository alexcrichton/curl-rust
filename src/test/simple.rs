use {get};
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

  let res = get("http://localhost:8482").unwrap();

  srv.assert();

  assert!(res.get_code() == 200, "code is {}", res.get_code());
  assert!(res.get_body() == "Hello".as_bytes());
  assert!(res.get_headers().len() == 1);
  assert!(res.get_header("Content-Length") == ["5".to_string()]);
}
