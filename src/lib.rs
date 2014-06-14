#![crate_id = "curl"]
#![feature(macro_rules)]

extern crate libc;
extern crate url;

use std::io;
use ffi::{easy,opt};
use body::Body;

pub use ffi::err::ErrCode;
pub use handle::{Handle,Request};
pub use response::{Headers,Response};

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
