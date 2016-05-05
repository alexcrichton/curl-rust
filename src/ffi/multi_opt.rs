#![allow(dead_code)]

use std::ffi::CString;
use std::path::Path;
use libc::{c_void};

use curl_ffi as ffi;

// multi
pub use curl_ffi::CURLMOPT_SOCKETFUNCTION as SOCKETFUNCTION;
pub use curl_ffi::CURLMOPT_SOCKETDATA as SOCKETDATA;

pub type Opt = ffi::CURLMoption;

pub trait OptVal {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void);
}

impl OptVal for isize {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self as *const c_void)
    }
}

impl OptVal for i32 {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        (self as isize).with_c_repr(f)
    }
}

impl OptVal for usize {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self as *const c_void)
    }
}

impl OptVal for bool {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self as usize as *const c_void)
    }
}

impl<'a> OptVal for &'a str {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        let s = CString::new(self).unwrap();
        f(s.as_ptr() as *const c_void)
    }
}

impl<'a> OptVal for &'a Path {
    #[cfg(unix)]
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        use std::ffi::OsStr;
        use std::os::unix::prelude::*;
        let s: &OsStr = self.as_ref();
        let s = CString::new(s.as_bytes()).unwrap();
        f(s.as_ptr() as *const c_void)
    }
    #[cfg(windows)]
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        let s = CString::new(self.to_str().unwrap()).unwrap();
        f(s.as_ptr() as *const c_void)
    }
}
