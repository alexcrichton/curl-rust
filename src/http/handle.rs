use std::collections::hash_map::{HashMap, Entry};
use std::path::Path;

use url::Url;

use ffi::easy::Easy;
use ffi::err;
use ffi::opt;
use ffi;
use http::Response;
use http::body::{Body,ToBody};
use {ProgressCb,ErrCode};

use self::Method::{Get, Head, Post, Put, Patch, Delete};
use self::BodyType::{Fixed, Chunked};

const DEFAULT_TIMEOUT_MS: usize = 30_000;

pub struct Handle {
    easy: Easy,
}

impl Handle {
    pub fn new() -> Handle {
        return configure(Handle { easy: Easy::new() }
            .timeout(DEFAULT_TIMEOUT_MS)
            .connect_timeout(DEFAULT_TIMEOUT_MS));

        #[cfg(all(unix, not(target_os = "macos")))]
        fn configure(mut handle: Handle) -> Handle {
            let probe = ::openssl::probe::probe();
            if let Some(ref path) = probe.cert_file {
                set_path(&mut handle, opt::CAINFO, path);
            }
            if let Some(ref path) = probe.cert_dir {
                set_path(&mut handle, opt::CAPATH, path);
            }
            return handle;

            fn set_path(handle: &mut Handle, opt: opt::Opt, path: &Path) {
                if let Err(e) = handle.easy.setopt(opt, path) {
                    if let err::NOT_BUILT_IN = e.code() {
                        return
                    }
                    panic!("failed to set {:?}: {}", opt, e)
                }
            }
        }

        #[cfg(any(not(unix), target_os = "macos"))]
        fn configure(handle: Handle) -> Handle { handle }
    }

    pub fn timeout(mut self, ms: usize) -> Handle {
        self.easy.setopt(opt::TIMEOUT_MS, ms).unwrap();
        self
    }

    pub fn connect_timeout(mut self, ms: usize) -> Handle {
        self.easy.setopt(opt::CONNECTTIMEOUT_MS, ms).unwrap();
        self
    }

    /// Set the time in seconds that the transfer speed should be below
    /// the `low_speed_limit` rate of bytes per second for the library to
    /// consider it too slow and abort.
    ///
    /// The default for this option is 0 which means that this option is
    /// disabled.
    pub fn low_speed_timeout(mut self, seconds: usize) -> Handle {
        self.easy.setopt(opt::LOW_SPEED_TIME, seconds).unwrap();
        self
    }

    /// Set the average transfer speed in bytes per second that the
    /// transfer should be below during `low_speed_timeout` seconds for
    /// libcurl to consider it to be too slow and abort.
    ///
    /// The default for this option is 0 which means that this option is
    /// disabled.
    pub fn low_speed_limit(mut self, bytes_per_second: usize) -> Handle {
        self.easy.setopt(opt::LOW_SPEED_LIMIT, bytes_per_second).unwrap();
        self
    }

    pub fn ssl_verifypeer(mut self, value: bool) -> Handle {
        self.easy.setopt(opt::SSL_VERIFYPEER, value).unwrap();
        self
    }

    pub fn follow_location(mut self, value: isize) -> Handle {
        self.easy.setopt(opt::FOLLOWLOCATION, value).unwrap();
        self
    }

    pub fn userpwd(mut self, userpwd: &str) -> Handle {
        self.easy.setopt(opt::USERPWD, userpwd).unwrap();
        self
    }

    pub fn verbose(mut self) -> Handle {
        self.easy.setopt(opt::VERBOSE, 1).unwrap();
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

    pub fn cookie_jar(mut self, path: &Path) -> Handle {
        self.easy.setopt(opt::COOKIEJAR, path).unwrap();
        self
    }

    pub fn cookie_file(mut self, path: &Path) -> Handle {
        self.easy.setopt(opt::COOKIEFILE, path).unwrap();
        self
    }

    pub fn cookies(self, path: &Path) -> Handle {
        self.cookie_jar(path).cookie_file(path)
    }

    pub fn cookie(mut self, cookie: &str) -> Handle {
        self.easy.setopt(opt::COOKIELIST, cookie).unwrap();
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

    pub fn patch<'a, 'b, U: ToUrl, B: ToBody<'b>>(&'a mut self, uri: U, body: B) -> Request<'a, 'b> {
        Request::new(self, Patch).uri(uri).body(body)
    }

    pub fn delete<'a, 'b, U: ToUrl>(&'a mut self, uri: U) -> Request<'a, 'b> {
        Request::new(self, Delete).uri(uri)
    }
}

#[derive(Copy, Clone)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Patch,
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
    progress: Option<Box<ProgressCb<'b>>>,
    follow: bool,
}

enum BodyType {
    Fixed(usize),
    Chunked,
}

impl<'a, 'b> Request<'a, 'b> {
    pub fn new(handle: &'a mut Handle, method: Method) -> Request<'a, 'b> {
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

    pub fn content_length(mut self, len: usize) -> Request<'a, 'b> {
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

    pub fn get_header(&self, name: &str) -> Option<&[String]> {
        self.headers.get(name).map(|a| &a[..])
    }

    pub fn headers<'c, 'd, I: Iterator<Item=(&'c str, &'d str)>>(mut self, hdrs: I) -> Request<'a, 'b> {
        for (name, val) in hdrs {
            append_header(&mut self.headers, name, val);
        }

        self
    }

    pub fn progress<F>(mut self, cb: F) -> Request<'a, 'b>
        where F: FnMut(usize, usize, usize, usize) + 'b
    {
        self.progress = Some(Box::new(cb) as Box<ProgressCb<'b>>);
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
            try!(handle.easy.setopt(opt::FOLLOWLOCATION, 1));
        }

        match err {
            Some(e) => return Err(e),
            None => {}
        }

        // Clear custom headers set from the previous request
        try!(handle.easy.setopt(opt::HTTPHEADER, 0));

        match method {
            Get => try!(handle.easy.setopt(opt::HTTPGET, 1)),
            Head => try!(handle.easy.setopt(opt::NOBODY, 1)),
            Post => try!(handle.easy.setopt(opt::POST, 1)),
            Put => try!(handle.easy.setopt(opt::UPLOAD, 1)),
            Patch => {
                try!(handle.easy.setopt(opt::CUSTOMREQUEST, "PATCH"));
                try!(handle.easy.setopt(opt::UPLOAD, 1));
            },
            Delete => {
                if body.is_some() {
                    try!(handle.easy.setopt(opt::UPLOAD, 1));
                }

                try!(handle.easy.setopt(opt::CUSTOMREQUEST, "DELETE"));
            }
            _ => unimplemented!()
        }

        match body.as_ref() {
            None => {}
            Some(body) => {
                let body_type = body_type.unwrap_or(match body.get_size() {
                    Some(len) => Fixed(len),
                    None => Chunked,
                });

                match body_type {
                    Fixed(len) => {
                        match method {
                            Post => try!(handle.easy.setopt(opt::POSTFIELDSIZE, len)),
                            Put | Patch | Delete  => try!(handle.easy.setopt(opt::INFILESIZE, len)),
                            _ => {
                                append_header(&mut headers, "Content-Length",
                                              &len.to_string());
                            }
                        }
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
                buf.extend(k.bytes());
                buf.extend(": ".bytes());

                for v in v.iter() {
                    buf.extend(v.bytes());
                    buf.push(0);
                    ffi_headers.push_bytes(&buf);
                    buf.truncate(k.len() + 2);
                }

                buf.truncate(0);
            }

            try!(handle.easy.setopt(opt::HTTPHEADER, &ffi_headers));
        }

        // According to libcurl's thread safety docs [1]:
        //
        // > When using multiple threads you should set the CURLOPT_NOSIGNAL
        // > option to 1L for all handles
        //
        // This library is likely to be used in a multithreaded situation, so
        // set this option as such. The implication of this is that timeouts are
        // not honored during the DNS lookup phase, but using c-ares can fix
        // that and it seems a small price to pay for thread safety!
        //
        // [1]: http://curl.haxx.se/libcurl/c/threadsafe.html
        try!(handle.easy.setopt(opt::NOSIGNAL, 1));

        handle.easy.perform(body.as_mut(), progress)
    }
}

fn append_header(map: &mut HashMap<String, Vec<String>>, key: &str, val: &str) {
    match map.entry(key.to_string()) {
        Entry::Vacant(entry) => {
            let mut values = Vec::new();
            values.push(val.to_string());
            entry.insert(values)
        },
        Entry::Occupied(entry) => entry.into_mut()
    };
}

pub trait ToUrl{
    fn with_url_str<F>(self, f: F) where F: FnOnce(&str);
}

impl<'a> ToUrl for &'a str {
    fn with_url_str<F>(self, f: F) where F: FnOnce(&str) {
        f(self);
    }
}

impl<'a> ToUrl for &'a Url {
    fn with_url_str<F>(self, f: F) where F: FnOnce(&str) {
        self.to_string().with_url_str(f);
    }
}

impl ToUrl for String {
    fn with_url_str<F>(self, f: F) where F: FnOnce(&str) {
        self[..].with_url_str(f);
    }
}

#[cfg(test)]
mod tests {
    use super::Handle;

    #[test]
    fn get_header() {
        let mut h = Handle::new();
        let r = h.get("/foo").header("foo", "bar");
        assert_eq!(r.get_header("foo"), Some(&["bar".to_string()][..]));
    }
}
