#![allow(bad_style)]

extern crate curl_sys;
extern crate libc;

use curl_sys::*;
use libc::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
