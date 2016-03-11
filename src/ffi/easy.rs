#![allow(dead_code)]

use std::sync::{Once, ONCE_INIT};
use std::mem;
use std::collections::HashMap;
use std::slice;
use libc::{self, c_int, c_long, c_double, size_t};
use super::{consts, err, info, opt};
use http::body::Body;
use http::{header, Response};

use curl_ffi as ffi;

pub type ProgressCb<'a> = FnMut(usize, usize, usize, usize) + 'a;

pub struct Easy {
    curl: *mut ffi::CURL
}

impl Easy {
    pub fn new() -> Easy {
        // Ensure that curl is globally initialized
        global_init();

        let handle = unsafe {
            let p = ffi::curl_easy_init();
            ffi::curl_easy_setopt(p, opt::NOPROGRESS, 0);
            p
        };

        Easy { curl: handle }
    }

    #[inline]
    pub fn setopt<T: opt::OptVal>(&mut self, option: opt::Opt, val: T) -> Result<(), err::ErrCode> {
        // TODO: Prevent setting callback related options
        let mut res = err::ErrCode(err::OK);

        unsafe {
            val.with_c_repr(|repr| {
                res = err::ErrCode(ffi::curl_easy_setopt(self.curl, option, repr));
            })
        }

        if res.is_success() { Ok(()) } else { Err(res) }
    }

    pub fn perform(&mut self,
                   body: Option<&mut Body>,
                   progress: Option<Box<ProgressCb>>)
                   -> Result<Response, err::ErrCode> {
        let mut builder = ResponseBuilder::new();

        unsafe {
            let resp_p: usize = mem::transmute(&builder);
            let body_p: usize = match body {
                Some(b) => mem::transmute(b),
                None => 0
            };

            let progress_p: usize = match progress.as_ref() {
                Some(cb) => mem::transmute(cb),
                None => 0
            };

            // Set callback options
            //
            // Use explicit `as` casts to work around rust-lang/rust#32201
            ffi::curl_easy_setopt(self.curl, opt::READFUNCTION,
                                  curl_read_fn as extern fn(_, _, _, _) -> _);
            ffi::curl_easy_setopt(self.curl, opt::READDATA, body_p);

            ffi::curl_easy_setopt(self.curl, opt::WRITEFUNCTION,
                                  curl_write_fn as extern fn(_, _, _, _) -> _);
            ffi::curl_easy_setopt(self.curl, opt::WRITEDATA, resp_p);

            ffi::curl_easy_setopt(self.curl, opt::HEADERFUNCTION,
                                  curl_header_fn as extern fn(_, _, _, _) -> _);
            ffi::curl_easy_setopt(self.curl, opt::HEADERDATA, resp_p);

            ffi::curl_easy_setopt(self.curl, opt::PROGRESSFUNCTION,
                                  curl_progress_fn as extern fn(_, _, _, _, _) -> _);
            ffi::curl_easy_setopt(self.curl, opt::PROGRESSDATA, progress_p);
        }

        let err = err::ErrCode(unsafe { ffi::curl_easy_perform(self.curl) });

        // If the request failed, abort here
        if !err.is_success() {
            return Err(err);
        }

        // Try to get the response code
        builder.code = try!(self.get_response_code());

        Ok(builder.build())
    }

    pub fn get_response_code(&self) -> Result<u32, err::ErrCode> {
        Ok(try!(self.get_info_long(info::RESPONSE_CODE)) as u32)
    }

    pub fn get_total_time(&self) -> Result<usize, err::ErrCode> {
        Ok(try!(self.get_info_long(info::TOTAL_TIME)) as usize)
    }

    fn get_info_long(&self, key: info::Key) -> Result<c_long, err::ErrCode> {
        let v: c_long = 0;
        let res = err::ErrCode(unsafe {
            ffi::curl_easy_getinfo(self.curl as *const _, key, &v)
        });

        if !res.is_success() {
            return Err(res);
        }

        Ok(v)
    }
}

#[inline]
fn global_init() {
    // Schedule curl to be cleaned up after we're done with this whole process
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| unsafe {
        assert_eq!(libc::atexit(cleanup), 0);
    });

    extern fn cleanup() {
        unsafe { ffi::curl_global_cleanup() }
    }
}

impl Drop for Easy {
    fn drop(&mut self) {
        unsafe { ffi::curl_easy_cleanup(self.curl) }
    }
}

/*
 *
 * TODO: Move this into handle
 *
 */

struct ResponseBuilder {
    code: u32,
    hdrs: HashMap<String,Vec<String>>,
    body: Vec<u8>
}

impl ResponseBuilder {
    fn new() -> ResponseBuilder {
        ResponseBuilder {
            code: 0,
            hdrs: HashMap::new(),
            body: Vec::new()
        }
    }

    fn add_header(&mut self, name: &str, val: &str) {
        // TODO: Reduce allocations
        use std::ascii::AsciiExt;
        let name = name.to_ascii_lowercase();

        let inserted = match self.hdrs.get_mut(&name) {
            Some(vals) => {
                vals.push(val.to_string());
                true
            }
            None => false
        };

        if !inserted {
            self.hdrs.insert(name, vec!(val.to_string()));
        }
    }

    fn build(self) -> Response {
        let ResponseBuilder { code, hdrs, body } = self;
        Response::new(code, hdrs, body)
    }
}

/*
 *
 * ===== Callbacks =====
 */

extern fn curl_read_fn(p: *mut u8, size: size_t, nmemb: size_t,
                       body: *mut Body) -> size_t {
    if body.is_null() {
        return 0;
    }

    let dst = unsafe { slice::from_raw_parts_mut(p, (size * nmemb) as usize) };
    let body = unsafe { &mut *body };

    match body.read(dst) {
        Ok(len) => len as size_t,
        Err(_) => consts::CURL_READFUNC_ABORT as size_t,
    }
}

extern fn curl_write_fn(p: *mut u8, size: size_t, nmemb: size_t,
                        resp: *mut ResponseBuilder) -> size_t {
    if !resp.is_null() {
        let builder: &mut ResponseBuilder = unsafe { mem::transmute(resp) };
        let chunk = unsafe { slice::from_raw_parts(p as *const u8,
                                                   (size * nmemb) as usize) };
        builder.body.extend(chunk.iter().map(|x| *x));
    }

    size * nmemb
}

extern fn curl_header_fn(p: *mut u8, size: size_t, nmemb: size_t,
                         resp: &mut ResponseBuilder) -> size_t {
    // TODO: Skip the first call (it seems to be the status line)

    let vec = unsafe { slice::from_raw_parts(p as *const u8,
                                             (size * nmemb) as usize) };

    match header::parse(&vec) {
        Some((name, val)) => {
            resp.add_header(name, val);
        }
        None => {}
    }

    vec.len() as size_t
}

pub extern "C" fn curl_progress_fn(cb: *mut Box<ProgressCb>, dltotal: c_double, dlnow: c_double, ultotal: c_double, ulnow: c_double) -> c_int {
    #[inline]
    fn to_usize(v: c_double) -> usize {
        if v > 0.0 { v as usize } else { 0 }
    }

    if !cb.is_null() {
        let cb: &mut ProgressCb = unsafe { &mut **cb };
        (*cb)(to_usize(dltotal), to_usize(dlnow), to_usize(ultotal), to_usize(ulnow));
    }

    0
}
