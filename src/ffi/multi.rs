#![allow(dead_code)]

use std::sync::{Once, ONCE_INIT};
use std::mem;
use std::collections::HashMap;
use std::slice;
use libc::{self, c_int, c_long, c_double, size_t};
use super::{consts, err, info, opt};
use super::multi_err::*;
use http::body::Body;
use http::{header, Response};
use super::easy::Easy;

use curl_ffi as ffi;

pub type ProgressCb<'a> = FnMut(usize, usize, usize, usize) + 'a;

pub struct Multi {
    curl: *mut ffi::CURLM
}

impl Multi {
    pub fn new() -> Multi {
        // Ensure that curl is globally initialized
        global_init();

        let handle = unsafe {
            let p = ffi::curl_multi_init();
            /* setup the generic multi interface options we want */
//            ffi::curl_multi_setopt(p, p, opt::SOCKETDATA, &p);
//            ffi::curl_multi_setopt(p, CURLMOPT_TIMERFUNCTION, curl_progress_fn);
//            ffi::curl_multi_setopt(p, CURLMOPT_TIMERDATA, &p);

//            ffi::curl_multi_setopt(p, opt::NOPROGRESS, 0);
            p
        };

        Multi { curl: handle }
    }

    pub fn add_connection(&mut self, easy: Easy) -> Result<(), ErrCodeM> {
        let mut res = ErrCodeM(OK);

        unsafe { res = ErrCodeM(ffi::curl_multi_add_handle(self.curl, easy.curl)); }

        if res.is_success() { Ok(()) } else { Err(res) }
    }

    #[inline]
    pub fn setopt<T: opt::OptVal>(&mut self, option: opt::Opt, val: T) -> Result<(), ErrCodeM> {
        // TODO: Prevent setting callback related options
        let mut res = ErrCodeM(OK);

        unsafe {
            val.with_c_repr(|repr| {
                res = ErrCodeM(ffi::curl_multi_setopt(self.curl, option, repr));
            })
        }

        if res.is_success() { Ok(()) } else { Err(res) }
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
        // TODO perhaps this and easy could check the cURL code.
        unsafe { ffi::curl_global_cleanup() }
    }
}

impl Drop for Multi {
    fn drop(&mut self) {
        unsafe { ffi::curl_multi_cleanup(self.curl) }
    }
}

/*
 *
 * TODO: Move this into handle
 *
 */

struct ResponseBuilderM {
    code: u32,
    hdrs: HashMap<String,Vec<String>>,
    body: Vec<u8>
}

impl ResponseBuilderM {
    fn new() -> ResponseBuilderM {
        ResponseBuilderM {
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
        let ResponseBuilderM { code, hdrs, body } = self;
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
                        resp: *mut ResponseBuilderM) -> size_t {
    if !resp.is_null() {
        let builder: &mut ResponseBuilderM = unsafe { mem::transmute(resp) };
        let chunk = unsafe { slice::from_raw_parts(p as *const u8,
                                                   (size * nmemb) as usize) };
        builder.body.extend(chunk.iter().map(|x| *x));
    }

    size * nmemb
}

extern fn curl_header_fn(p: *mut u8, size: size_t, nmemb: size_t,
                         resp: &mut ResponseBuilderM) -> size_t {
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
