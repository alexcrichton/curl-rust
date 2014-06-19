use std::io::{BufReader,IoResult,Reader};

pub enum Body<'a> {
  FixedBody(BufReader<'a>, uint),
  ChunkedBody(&'a mut Reader)
}

impl<'a> Body<'a> {
  pub fn get_size(&self) -> Option<uint> {
    match self {
      &FixedBody(b, len) => Some(len),
      _ => None
    }
  }

  pub fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
    match self {
      &FixedBody(ref mut r,_) => r.read(buf),
      &ChunkedBody(ref mut r) => r.read(buf)
    }
  }
}

pub trait ToBody<'a> {
  fn to_body(self) -> Body<'a>;
}

impl<'a> ToBody<'a> for &'a str {
  fn to_body(self) -> Body<'a> {
    self.as_bytes().to_body()
  }
}

impl<'a> ToBody<'a> for &'a [u8] {
  fn to_body(self) -> Body<'a> {
    FixedBody(BufReader::new(self), self.len())
  }
}

impl<'a> ToBody<'a> for &'a String {
  fn to_body(self) -> Body<'a> {
    self.as_slice().to_body()
  }
}

// TODO: https://github.com/rust-lang/rust/issues/14901
impl<'a> ToBody<'a> for &'a mut Reader {
  fn to_body(self) -> Body<'a> {
    ChunkedBody(self)
  }
}
