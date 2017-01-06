//! Rust bindings to the libcurl C library
//!
//! This crate contains bindings for an HTTP/HTTPS client which is powered by
//! [libcurl], the same library behind the `curl` command line tool. The API
//! currently closely matches that of libcurl itself, except that a Rustic layer
//! of safety is applied on top.
//!
//! [libcurl]: https://curl.haxx.se/libcurl/
//!
//! # The "Easy" API
//!
//! The easiest way to send a request is to use the `Easy` api which corresponds
//! to `CURL` in libcurl. This handle supports a wide variety of options and can
//! be used to make a single blocking request in a thread. Callbacks can be
//! specified to deal with data as it arrives and a handle can be reused to
//! cache connections and such.
//!
//! ```rust,no_run
//! use std::io::{stdout, Write};
//!
//! use curl::easy::Easy;
//!
//! // Write the contents of rust-lang.org to stdout
//! let mut easy = Easy::new();
//! easy.url("https://www.rust-lang.org/").unwrap();
//! easy.write_function(|data| {
//!     Ok(stdout().write(data).unwrap())
//! }).unwrap();
//! easy.perform().unwrap();
//! ```
//!
//! # What about multiple concurrent HTTP requests?
//!
//! One option you have currently is to send multiple requests in multiple
//! threads, but otherwise libcurl has a "multi" interface for doing this
//! operation. Initial bindings of this interface can be found in the `multi`
//! module, but feedback is welcome!
//!
//! # Where does libcurl come from?
//!
//! This crate links to the `curl-sys` crate which is in turn responsible for
//! acquiring and linking to the libcurl library. Currently this crate will
//! build libcurl from source if one is not already detected on the system.
//!
//! There is a large number of releases for libcurl, all with different sets of
//! capabilities. Robust programs may wish to inspect `Version::get()` to test
//! what features are implemented in the linked build of libcurl at runtime.

#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/curl/0.4")]

extern crate curl_sys;
extern crate libc;

#[cfg(all(unix, not(target_os = "macos")))]
extern crate openssl_sys;
#[cfg(all(unix, not(target_os = "macos")))]
extern crate openssl_probe;
#[cfg(windows)]
extern crate winapi;

use std::ffi::CStr;
use std::str;
use std::sync::{Once, ONCE_INIT};

pub use error::{Error, ShareError, MultiError, FormError};
mod error;

pub use version::{Version, Protocols};
mod version;

mod panic;
pub mod easy;
pub mod multi;

/// Initializes the underlying libcurl library.
///
/// It's not required to call this before the library is used, but it's
/// recommended to do so as soon as the program starts.
pub fn init() {
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| {
        platform_init();
        unsafe {
            assert_eq!(curl_sys::curl_global_init(curl_sys::CURL_GLOBAL_ALL), 0);
        }

        // Note that we explicitly don't schedule a call to
        // `curl_global_cleanup`. The documentation for that function says
        //
        // > You must not call it when any other thread in the program (i.e. a
        // > thread sharing the same memory) is running. This doesn't just mean
        // > no other thread that is using libcurl.
        //
        // We can't ever be sure of that, so unfortunately we can't call the
        // function.
    });

    #[cfg(all(unix, not(target_os = "macos")))]
    fn platform_init() {
        openssl_sys::init();
    }

    #[cfg(not(all(unix, not(target_os = "macos"))))]
    fn platform_init() {}
}

unsafe fn opt_str<'a>(ptr: *const libc::c_char) -> Option<&'a str> {
    if ptr.is_null() {
        None
    } else {
        Some(str::from_utf8(CStr::from_ptr(ptr).to_bytes()).unwrap())
    }
}

fn cvt(r: curl_sys::CURLcode) -> Result<(), Error> {
    if r == curl_sys::CURLE_OK {
        Ok(())
    } else {
        Err(Error::new(r))
    }
}
