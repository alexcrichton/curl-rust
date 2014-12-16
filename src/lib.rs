#![crate_name = "curl"]
#![feature(macro_rules)]
#![feature(phase)]

extern crate libc;
extern crate url;
#[cfg(test)] #[phase(plugin, link)] extern crate log;

extern crate "curl-sys" as curl_ffi;
#[cfg(unix)] extern crate "openssl-sys" as openssl;

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

#[cfg(test)]
mod test;
