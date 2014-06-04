use std::{fmt,str};
use collections::HashMap;

pub type Headers = HashMap<String, Vec<String>>;

pub struct Response {
  code: uint,
  hdrs: Headers,
  body: Vec<u8>
}

impl Response {
  pub fn new(code: uint, hdrs: Headers, body: Vec<u8>) -> Response {
    Response {
      code: code,
      hdrs: hdrs,
      body: body
    }
  }

  pub fn get_code(&self) -> uint {
    self.code
  }

  pub fn get_header<'a>(&'a self, name: &str) -> &'a [&'a str] {
    unimplemented!()
  }

  pub fn get_body<'a>(&'a self) -> &'a [u8] {
    self.body.as_slice()
  }
}

impl fmt::Show for Response {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
    try!(write!(fmt, "Response \\{ {}, ", self.code));

    for (name, val) in self.hdrs.iter() {
      try!(write!(fmt, "{}: {}, ", name, val.connect(", ")));
    }

    match str::from_utf8(self.body.as_slice()) {
      Some(b) => try!(write!(fmt, "{}", b)),
      None => try!(write!(fmt, "bytes[{}]", self.body.len()))
    }

    Ok(())
  }
}
