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
//!     stdout().write_all(data).unwrap();
//!     Ok(data.len())
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
//!
//! # Initialization
//!
//! The underlying libcurl library must be initialized before use and has
//! certain requirements on how this is done. Check the documentation for
//! [`init`] for more details.

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/curl/0.4")]

extern crate curl_sys;
extern crate libc;
extern crate socket2;

#[cfg(need_openssl_probe)]
extern crate openssl_probe;
#[cfg(need_openssl_init)]
extern crate openssl_sys;

#[cfg(target_env = "msvc")]
extern crate schannel;

use std::ffi::CStr;
use std::str;
use std::sync::Once;

pub use error::{Error, FormError, MultiError, ShareError};
mod error;

pub use version::{Protocols, Version};
mod version;

pub mod easy;
pub mod multi;
mod panic;

/// Initializes the underlying libcurl library.
///
/// The underlying libcurl library must be initialized before use, and must be
/// done so on the main thread before any other threads are created by the
/// program. This crate will do this for you automatically in the following
/// scenarios:
///
/// - Creating a new [`Easy`][easy::Easy] or [`Multi`][multi::Multi] handle
/// - At program startup on Windows, macOS, Linux, Android, or FreeBSD systems
///
/// This should be sufficient for most applications and scenarios, but in any
/// other case, it is strongly recommended that you call this function manually
/// as soon as your program starts.
///
/// Calling this function more than once is harmless and has no effect.
#[inline]
pub fn init() {
    /// Used to prevent concurrent or duplicate initialization.
    static INIT: Once = Once::new();

    /// An exported constructor function. On supported platforms, this will be
    /// invoked automatically before the program's `main` is called.
    #[cfg_attr(
        any(target_os = "linux", target_os = "freebsd", target_os = "android"),
        link_section = ".init_array"
    )]
    #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
    #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
    static INIT_CTOR: extern "C" fn() = init_inner;

    /// This is the body of our constructor function.
    #[cfg_attr(
        any(target_os = "linux", target_os = "android"),
        link_section = ".text.startup"
    )]
    extern "C" fn init_inner() {
        INIT.call_once(|| {
            #[cfg(need_openssl_init)]
            openssl_sys::init();

            unsafe {
                assert_eq!(curl_sys::curl_global_init(curl_sys::CURL_GLOBAL_ALL), 0);
            }

            // Note that we explicitly don't schedule a call to
            // `curl_global_cleanup`. The documentation for that function says
            //
            // > You must not call it when any other thread in the program (i.e.
            // > a thread sharing the same memory) is running. This doesn't just
            // > mean no other thread that is using libcurl.
            //
            // We can't ever be sure of that, so unfortunately we can't call the
            // function.
        });
    }

    // We invoke our init function through our static to ensure the symbol isn't
    // optimized away by a bug: https://github.com/rust-lang/rust/issues/47384
    INIT_CTOR();
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
