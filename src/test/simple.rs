use std::io::stdio::stdout;
use {get,request};
use super::server;

#[test]
pub fn test_simple_get() {
  let srv = server!(
    recv!("FOO"), // Send the data
    send!("FOO")); // Sends

  let res = get("http://localhost:8482");
  stdout().write_str("Finishing request\n");

  stdout().write_str(format!("res: {}", res).as_slice());

  srv.assert();
  stdout().write_str("Finished test\n");
  // assert!(srv.recv());
  // assert!(res.is_success());

  // fail!("nope");
}
