use std::c_vec::CVec;
use std::{mem,str};
use libc::{c_void,c_long,size_t};
use super::{info,opt};
use super::err::ErrCode;
use {header,Response};

pub type CURL = c_void;

#[link(name = "curl")]
extern {
  pub fn curl_easy_init() -> *CURL;
  pub fn curl_easy_setopt(curl: *CURL, option: opt::Opt, ...) -> ErrCode;
  pub fn curl_easy_perform(curl: *CURL) -> ErrCode;
  pub fn curl_easy_cleanup(curl: *CURL);
  pub fn curl_easy_getinfo(curl: *CURL, info: info::Key, ...) -> ErrCode;
}
