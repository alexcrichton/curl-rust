//! TODO: dox

#![deny(missing_docs)]

// extern crate libc;
// extern crate url;

extern crate curl_sys;

// #[cfg(all(unix, not(target_os = "macos")))]
// extern crate openssl_sys as openssl;
//
// pub use ffi::easy::ProgressCb;
// pub use ffi::err::ErrCode;
//
// pub use error::Error;
//
// // Version accessors
// pub use ffi::version::{
//     Version,
//     version,
//     version_info,
//     Protocols
// };

pub use error::{Error, ShareError, MultiError};
mod error;
// mod ffi;
// pub mod http;
