#![crate_name = "curl"]
#![feature(macro_rules)]
#![feature(phase)]

extern crate libc;
extern crate url;

#[phase(plugin, link)] extern crate log;
#[phase(plugin, link)] extern crate "link-config" as link_config;

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
