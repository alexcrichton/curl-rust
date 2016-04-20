//! TODO: dox

#![deny(missing_docs)]

extern crate curl_sys;
extern crate libc;

use std::ffi::CStr;
use std::str;
use std::sync::{Once, ONCE_INIT};

pub use error::{Error, ShareError, MultiError};
mod error;

pub use version::{Version, Protocols};
mod version;

mod panic;
pub mod easy;

/// Initializes the underlying libcurl library.
///
/// It's not required to call this before the library is used, but it's
/// recommended to do so as soon as the program starts.
pub fn init() {
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| {
        unsafe {
            curl_sys::curl_global_init(curl_sys::CURL_GLOBAL_ALL);
            libc::atexit(cleanup);
        }
    });

    extern fn cleanup() {
        unsafe { curl_sys::curl_global_cleanup(); }
    }
}

unsafe fn opt_str<'a>(ptr: *const libc::c_char) -> Option<&'a str> {
    if ptr.is_null() {
        None
    } else {
        Some(str::from_utf8(CStr::from_ptr(ptr).to_bytes()).unwrap())
    }
}
