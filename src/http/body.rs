use std::io::{BufReader,IoResult,Reader};

use self::Body::{FixedBody, ChunkedBody};

pub enum Body<'a> {
    FixedBody(BufReader<'a>, uint),
    ChunkedBody(&'a mut Reader+'a)
}

impl<'a> Body<'a> {
    pub fn get_size(&self) -> Option<uint> {
        match self {
            &FixedBody(_, len) => Some(len),
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

impl<'a, R: 'a+Reader> ToBody<'a> for &'a mut R {
    fn to_body(self) -> Body<'a> {
        ChunkedBody(self)
    }
}
