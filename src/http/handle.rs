use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};
use std::path::Path;
use url::Url;

use ffi;
use ffi::opt;
use ffi::easy::Easy;
use http::Response;
use http::body::{Body,ToBody};
use {ProgressCb,ErrCode};

static DEFAULT_TIMEOUT_MS: uint = 30_000;

pub struct Handle {
    easy: Easy,
}

impl Handle {
    pub fn new() -> Handle {
        Handle { easy: Easy::new() }
            .timeout(DEFAULT_TIMEOUT_MS)
            .connect_timeout(DEFAULT_TIMEOUT_MS)
    }

    pub fn timeout(mut self, ms: uint) -> Handle {
        self.easy.setopt(opt::TIMEOUT_MS, ms).unwrap();
        self
    }

    pub fn connect_timeout(mut self, ms: uint) -> Handle {
        self.easy.setopt(opt::CONNECTTIMEOUT_MS, ms).unwrap();
        self
    }

    pub fn verbose(mut self) -> Handle {
        self.easy.setopt(opt::VERBOSE, 1u).unwrap();
        self
    }

    pub fn proxy<U: ToUrl>(mut self, proxy: U) -> Handle {
        proxy.with_url_str(|s| {
            self.easy.setopt(opt::PROXY, s).unwrap();
        });

        self
    }

    pub fn ssl_ca_path(mut self, path: &Path) -> Handle {
        self.easy.setopt(opt::CAPATH, path).unwrap();
        self
    }

    pub fn ssl_ca_info(mut self, path: &Path) -> Handle {
        self.easy.setopt(opt::CAINFO, path).unwrap();
        self
    }

    pub fn get<'a, 'b, U: ToUrl>(&'a mut self, uri: U) -> Request<'a, 'b> {
        Request::new(self, Get).uri(uri)
    }

    pub fn head<'a, 'b, U: ToUrl>(&'a mut self, uri: U) -> Request<'a, 'b> {
        Request::new(self, Head).uri(uri)
    }

    pub fn post<'a, 'b, U: ToUrl, B: ToBody<'b>>(&'a mut self, uri: U, body: B) -> Request<'a, 'b> {
        Request::new(self, Post).uri(uri).body(body)
    }

    pub fn put<'a, 'b, U: ToUrl, B: ToBody<'b>>(&'a mut self, uri: U, body: B) -> Request<'a, 'b> {
        Request::new(self, Put).uri(uri).body(body)
    }

    pub fn delete<'a, 'b, U: ToUrl>(&'a mut self, uri: U) -> Request<'a, 'b> {
        Request::new(self, Delete).uri(uri)
    }
}

pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect
}

pub struct Request<'a, 'b> {
    err: Option<ErrCode>,
    handle: &'a mut Handle,
    method: Method,
    headers: HashMap<String, Vec<String>>,
    body: Option<Body<'b>>,
    body_type: Option<BodyType>,
    content_type: bool, // whether or not the content type was set
    expect_continue: bool, // whether to expect a 100 continue from the server
    progress: Option<ProgressCb<'b>>,
    follow: bool,
}

enum BodyType {
    Fixed(uint),
    Chunked,
}

impl<'a, 'b> Request<'a, 'b> {
    fn new<'a, 'b>(handle: &'a mut Handle, method: Method) -> Request<'a, 'b> {
        Request {
            err: None,
            handle: handle,
            method: method,
            headers: HashMap::new(),
            body: None,
            body_type: None,
            content_type: false,
            expect_continue: false,
            progress: None,
            follow: false,
        }
    }

    pub fn uri<U: ToUrl>(mut self, uri: U) -> Request<'a, 'b> {
        uri.with_url_str(|s| {
            match self.handle.easy.setopt(opt::URL, s) {
                Ok(_) => {}
                Err(e) => self.err = Some(e)
            }
        });

        self
    }

    pub fn body<B: ToBody<'b>>(mut self, body: B) -> Request<'a, 'b> {
        self.body = Some(body.to_body());
        self
    }

    pub fn content_type(mut self, ty: &str) -> Request<'a, 'b> {
        if !self.content_type {
            self.content_type = true;
            append_header(&mut self.headers, "Content-Type", ty);
        }

        self
    }

    pub fn content_length(mut self, len: uint) -> Request<'a, 'b> {
        self.body_type = Some(Fixed(len));
        self
    }

    pub fn chunked(mut self) -> Request<'a, 'b> {
        self.body_type = Some(Chunked);
        self
    }

    pub fn expect_continue(mut self) -> Request<'a, 'b> {
        self.expect_continue = true;
        self
    }

    pub fn header(mut self, name: &str, val: &str) -> Request<'a, 'b> {
        append_header(&mut self.headers, name, val);
        self
    }

    pub fn get_header<'a>(&'a self, name: &str) -> Option<&'a [String]> {
        self.headers.find_equiv(&name).map(|a| a.as_slice())
    }

    pub fn headers<'c, 'd, I: Iterator<(&'c str, &'d str)>>(mut self, mut hdrs: I) -> Request<'a, 'b> {
        for (name, val) in hdrs {
            append_header(&mut self.headers, name, val);
        }

        self
    }

    pub fn progress(mut self, cb: ProgressCb<'b>) -> Request<'a, 'b> {
        self.progress = Some(cb);
        self
    }

    pub fn follow_redirects(mut self, follow: bool) -> Request<'a, 'b> {
        self.follow = follow;
        self
    }

    pub fn exec(self) -> Result<Response, ErrCode> {
        // Deconstruct the struct
        let Request {
            err,
            handle,
            method,
            mut headers,
            mut body,
            body_type,
            content_type,
            expect_continue,
            progress,
            follow,
            ..
        } = self;

        if follow {
            try!(handle.easy.setopt(opt::FOLLOWLOCATION, 1i));
        }

        match err {
            Some(e) => return Err(e),
            None => {}
        }

        // Clear custom headers set from the previous request
        try!(handle.easy.setopt(opt::HTTPHEADER, 0u));

        match method {
            Get => try!(handle.easy.setopt(opt::HTTPGET, 1i)),
            Head => try!(handle.easy.setopt(opt::NOBODY, 1i)),
            Post => try!(handle.easy.setopt(opt::POST, 1i)),
            Put => try!(handle.easy.setopt(opt::UPLOAD, 1i)),
            Delete => {
                if body.is_some() {
                    try!(handle.easy.setopt(opt::UPLOAD, 1i));
                }

                try!(handle.easy.setopt(opt::CUSTOMREQUEST, "DELETE"));
            }
            _ => unimplemented!()
        }

        match body.as_ref() {
            None => {}
            Some(body) => {
                debug!("handling body");

                let body_type = body_type.unwrap_or(match body.get_size() {
                    Some(len) => Fixed(len),
                    None => Chunked,
                });

                match body_type {
                    Fixed(len) => {
                        match method {
                            Post => try!(handle.easy.setopt(opt::POSTFIELDSIZE, len)),
                            Put | Delete  => try!(handle.easy.setopt(opt::INFILESIZE, len)),
                            _ => {}
                        }
                        append_header(&mut headers, "Content-Length",
                                      len.to_string().as_slice());
                    }
                    Chunked => {
                        append_header(&mut headers, "Transfer-Encoding",
                                      "chunked");
                    }

                }

                if !content_type {
                    append_header(&mut headers, "Content-Type", "application/octet-stream");
                }

                if !expect_continue {
                    append_header(&mut headers, "Expect", "");
                }
            }
        }

        let mut ffi_headers = ffi::List::new();

        if !headers.is_empty() {
            let mut buf = Vec::new();

            for (k, v) in headers.iter() {
                buf.push_all(k.as_bytes());
                buf.push_all(b": ");

                for v in v.iter() {
                    buf.push_all(v.as_bytes());
                    buf.push(0);
                    ffi_headers.push_bytes(buf.as_slice());
                    buf.truncate(k.len() + 2);
                }

                buf.truncate(0);
            }

            try!(handle.easy.setopt(opt::HTTPHEADER, &ffi_headers));
        }

        handle.easy.perform(body.as_mut(), progress)
    }
}

fn append_header(map: &mut HashMap<String, Vec<String>>, key: &str, val: &str) {
    match map.entry(key.to_string()) {
        Vacant(entry) => {
            let mut values = Vec::new();
            values.push(val.to_string());
            entry.set(values)
        },
        Occupied(entry) => entry.into_mut()
    };
}

pub trait ToUrl{
    fn with_url_str(self, f: |&str|);
}

impl<'a> ToUrl for &'a str {
    fn with_url_str(self, f: |&str|) {
        f(self);
    }
}

impl<'a> ToUrl for &'a Url {
    fn with_url_str(self, f: |&str|) {
        self.to_string().with_url_str(f);
    }
}

impl ToUrl for String {
    fn with_url_str(self, f: |&str|) {
        self.as_slice().with_url_str(f);
    }
}

#[cfg(test)]
mod tests {
    use super::Handle;

    #[test]
    fn get_header() {
        let mut h = Handle::new();
        let r = h.get("/foo").header("foo", "bar");
        assert_eq!(r.get_header("foo"), Some(["bar".to_string()].as_slice()));
    }
}
