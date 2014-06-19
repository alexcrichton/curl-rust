use {handle};
use super::server;

#[test]
pub fn test_simple_head() {
  let srv = server!(
    recv!(
      "HEAD / HTTP/1.1\r\n\
       Host: localhost:8482\r\n\
       Accept: */*\r\n\
       \r\n"), // Send the data
    send!(
      "HTTP/1.1 200 OK\r\n\
       Content-Length: 5\r\n\r\n"));

  let res = handle()
    .head("http://localhost:8482")
    .exec();

  srv.assert();
  let res = res.unwrap();

  assert!(res.get_code() == 200, "code is {}", res.get_code());
  assert!(res.get_body() == []);
  assert!(res.get_headers().len() == 1);
  assert!(res.get_header("Content-Length") == ["5".to_string()]);
}
