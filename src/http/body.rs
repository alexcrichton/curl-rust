use std::io::prelude::*;
use std::io;

use self::Body::{FixedBody, ChunkedBody};

pub enum Body<'a> {
    FixedBody(&'a [u8], usize),
    ChunkedBody(&'a mut (Read+'a))
}

impl<'a> Body<'a> {
    pub fn get_size(&self) -> Option<usize> {
        match self {
            &FixedBody(_, len) => Some(len),
            _ => None
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            &mut FixedBody(ref mut r, _) => Read::read(r, buf),
            &mut ChunkedBody(ref mut r) => r.read(buf)
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
        FixedBody(self, self.len())
    }
}

impl<'a> ToBody<'a> for &'a String {
    fn to_body(self) -> Body<'a> {
        self[..].to_body()
    }
}

impl<'a, R: Read + 'a> ToBody<'a> for &'a mut R {
    fn to_body(self) -> Body<'a> {
        ChunkedBody(self)
    }
}
