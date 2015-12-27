use std::{fmt,str};
use std::collections::HashMap;

pub type Headers = HashMap<String, Vec<String>>;
pub type ResponseBody = Option<Vec<u8>>;

#[derive(Debug)]
pub struct Response {
    code: u32,
    hdrs: Headers,
    body: ResponseBody
}

impl Response {
    pub fn new(code: u32, hdrs: Headers, mut body: ResponseBody, allow_null_body: bool) -> Response {
        if !allow_null_body && body.is_none(){
            body = Some(Vec::new())
        }
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

    pub fn get_body<'a>(&'a self) -> &ResponseBody {
        &self.body
    }

    pub fn move_body(self) -> ResponseBody {
        self.body
    }
}

impl fmt::Display for Response {
    #[allow(deprecated)] // needed for `connect()`, since Rust 1.1 is supported
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "Response {{{}, ", self.code));

        for (name, val) in self.hdrs.iter() {
            try!(write!(fmt, "{}: {}, ", name, val.connect(", ")));
        }

        match self.body {
            Some(ref body) => match str::from_utf8(&body) {
                Ok(b) => try!(write!(fmt, "{}", b)),
                Err(..) => try!(write!(fmt, "bytes[{}]", body.len()))
            },
            None => try!(write!(fmt, "NoBody")),
        }

        try!(write!(fmt, "]"));

        Ok(())
    }
}
