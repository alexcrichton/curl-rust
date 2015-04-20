use std::{fmt,str};
use std::collections::HashMap;

pub type Headers = HashMap<String, Vec<String>>;

pub struct Response {
    code: u32,
    hdrs: Headers,
    body: Vec<u8>
}

impl Response {
    pub fn new(code: u32, hdrs: Headers, body: Vec<u8>) -> Response {
        Response {
            code: code,
            hdrs: hdrs,
            body: body
        }
    }

    pub fn get_code(&self) -> u32 {
        self.code
    }

    pub fn get_headers<'a>(&'a self) -> &'a Headers {
        &self.hdrs
    }

    pub fn get_header<'a>(&'a self, name: &str) -> &'a [String] {
        self.hdrs
            .get(name)
            .map(|v| &v[..])
            .unwrap_or(&[])
    }

    pub fn get_body<'a>(&'a self) -> &'a [u8] {
        &self.body
    }

    pub fn move_body(self) -> Vec<u8> {
        self.body
    }
}

impl fmt::Display for Response {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "Response {{{}, ", self.code));

        for (name, val) in self.hdrs.iter() {
            try!(write!(fmt, "{}: {}, ", name, val.connect(", ")));
        }

        match str::from_utf8(&self.body) {
            Ok(b) => try!(write!(fmt, "{}", b)),
            Err(..) => try!(write!(fmt, "bytes[{}]", self.body.len()))
        }

        try!(write!(fmt, "]"));

        Ok(())
    }
}
