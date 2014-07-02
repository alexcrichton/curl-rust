#![crate_id = "curl"]
#![feature(macro_rules)]
#![feature(phase)]

extern crate libc;
extern crate url;

#[phase(plugin, link)]
extern crate log;

pub use ffi::easy::ProgressCb;
pub use ffi::err::ErrCode;
pub use handle::{Handle,Request};
pub use response::{Headers,Response};

// Version accessors
pub use ffi::version::{
  Version,
  version,
  version_info,
  Protocols
};

mod body;
mod ffi;
mod handle;
mod header;
mod response;

#[cfg(test)]
mod test;

#[inline]
pub fn handle() -> Handle {
  Handle::new()
}
