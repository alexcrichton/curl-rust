#![crate_name = "curl"]
#![feature(macro_rules)]
#![feature(phase)]

extern crate libc;

#[phase(plugin, link)]
extern crate log;

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
