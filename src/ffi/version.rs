use std::c_str::CString;
use libc::{c_char,c_int,c_uint,c_long};

#[repr(C)]
pub enum CURLversion {
  CURLVERSION_FIRST,
  CURLVERSION_SECOND,
  CURLVERSION_THIRD,
  CURLVERSION_FOURTH,
  CURLVERSION_LAST /* never actually use this */
}

pub static CURLVERSION_NOW: CURLversion = CURLVERSION_FOURTH;

pub struct curl_version_info_data {
  pub age: CURLversion,
  pub version: *c_char,
  pub version_num: c_uint,
  pub host: *c_char,
  pub features: c_int,
  pub ssl_version: *c_char,
  pub ssl_version_num: c_long,
  pub libz_version: *c_char,

  /* protocols is terminated by an entry with a NULL protoname */
  pub protocols: **c_char,

  /* The fields below this were added in CURLVERSION_SECOND */
  pub ares: *c_char,
  pub ares_num: c_int,

  /* This field was added in CURLVERSION_THIRD */
  pub libidn: *c_char,

  /* These field were added in CURLVERSION_FOURTH */
  pub iconv_ver_num: c_int,
  pub libssh_version: *c_char,
}

impl curl_version_info_data {
  pub fn get_version_major(&self) -> uint {
    (self.version_num as uint & 0xFF0000) >> 16
  }
}

#[link(name = "curl")]
extern {
  fn curl_version() -> *c_char;
  pub fn curl_version_info(t: CURLversion) -> &'static curl_version_info_data;
}

pub fn version() -> String {
  let v = unsafe { CString::new(curl_version(), false) };
  v.as_str().unwrap().to_string()
}
