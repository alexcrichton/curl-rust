#![crate_id = "curl"]
#![feature(macro_rules)]

extern crate libc;
extern crate url;

pub use ffi::err::ErrCode;
pub use handle::{Handle,Request};
pub use response::{Headers,Response};

// Version accessors
pub use ffi::version::{
  version,
  ssl_version
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
