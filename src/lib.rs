extern crate libc;
extern crate url;

#[macro_use]
extern crate log;

extern crate curl_sys as curl_ffi;

#[cfg(unix)]
extern crate openssl_sys as openssl;

pub use ffi::easy::ProgressCb;
pub use ffi::err::ErrCode;

// Version accessors
pub use ffi::version::{
    Version,
    version,
    version_info,
    Protocols
};

mod ffi;
pub mod http;
