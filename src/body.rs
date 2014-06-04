use std::io::{IoResult,Reader};

pub struct Body<'a> {
  reader: &'a mut Reader
}

impl<'a> Body<'a> {
  pub fn new<'b>(reader: &'b mut Reader) -> Body<'b> {
    Body { reader: reader }
  }

  #[inline]
  pub fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
    self.reader.read(buf)
  }
}
