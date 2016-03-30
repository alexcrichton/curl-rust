#![allow(bad_style)]

extern crate curl_sys;
extern crate libc;

use libc::*;
use curl_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
