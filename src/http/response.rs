use std::{fmt,str};
use std::collections::HashMap;

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

    pub fn get_headers<'a>(&'a self) -> &'a Headers {
        &self.hdrs
    }

    pub fn get_header<'a>(&'a self, name: &str) -> &'a [String] {
        self.hdrs
            .find_equiv(&name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_body<'a>(&'a self) -> &'a [u8] {
        self.body.as_slice()
    }

    pub fn move_body(self) -> Vec<u8> {
        self.body
    }
}

impl fmt::Show for Response {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        try!(write!(fmt, "Response {{{}, ", self.code));

        for (name, val) in self.hdrs.iter() {
            try!(write!(fmt, "{}: {}, ", name, val.connect(", ")));
        }

        match str::from_utf8(self.body.as_slice()) {
            Some(b) => try!(write!(fmt, "{}", b)),
            None => try!(write!(fmt, "bytes[{}]", self.body.len()))
        }

        try!(write!(fmt, "]"));

        Ok(())
    }
}
