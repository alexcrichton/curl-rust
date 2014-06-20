#![allow(non_camel_case_types)]

use std::c_str::CString;
use std::{fmt,mem,ptr};
use libc::{c_char,c_int,c_uint,c_long};

#[repr(C)]
enum CURLversion {
  CURL_VERSION_FIRST,
  CURL_VERSION_SECOND,
  CURL_VERSION_THIRD,
  CURL_VERSION_FOURTH,
  CURL_VERSION_LAST /* never actually use this */
}

static CURL_VERSION_NOW: CURLversion    = CURL_VERSION_FOURTH;
static CURL_VERSION_IPV6:         c_int = (1 << 0);
static CURL_VERSION_KERBEROS4:    c_int = (1 << 1);
static CURL_VERSION_SSL:          c_int = (1 << 2);
static CURL_VERSION_LIBZ:         c_int = (1 << 3);
static CURL_VERSION_NTLM:         c_int = (1 << 4);
static CURL_VERSION_GSSNEGOTIATE: c_int = (1 << 5);
static CURL_VERSION_DEBUG:        c_int = (1 << 6);
static CURL_VERSION_ASYNCHDNS:    c_int = (1 << 7);
static CURL_VERSION_SPNEGO:       c_int = (1 << 8);
static CURL_VERSION_LARGEFILE:    c_int = (1 << 9);
static CURL_VERSION_IDN:          c_int = (1 << 10);
static CURL_VERSION_SSPI:         c_int = (1 << 11);
static CURL_VERSION_CONV:         c_int = (1 << 12);
static CURL_VERSION_CURLDEBUG:    c_int = (1 << 13);
static CURL_VERSION_TLSAUTH_SRP:  c_int = (1 << 14);
static CURL_VERSION_NTLM_WB:      c_int = (1 << 15);
static CURL_VERSION_HTTP2:        c_int = (1 << 16);

struct curl_version_info_data {
  #[allow(dead_code)]
  age: CURLversion,

  version: *c_char,
  version_num: c_uint,
  host: *c_char,
  features: c_int,
  ssl_version: *c_char,

  #[allow(dead_code)]
  ssl_version_num: c_long,

  libz_version: *c_char,

  /* protocols is terminated by an entry with a NULL protoname */
  protocols: **c_char,

  /* The fields below this were added in CURL_VERSION_SECOND */
  ares: *c_char,
  ares_num: c_int,

  /* This field was added in CURL_VERSION_THIRD */
  libidn: *c_char,

  /* These field were added in CURL_VERSION_FOURTH */
  iconv_ver_num: c_int,
  libssh_version: *c_char,
}

pub type Version = curl_version_info_data;

impl curl_version_info_data {

  pub fn version_str<'a>(&'a self) -> &'a str {
    as_str(self.version)
  }

  pub fn version_major(&self) -> uint {
    (self.version_num as uint & 0xFF0000) >> 16
  }

  pub fn version_minor(&self) -> uint {
    (self.version_num as uint & 0xFF00) >> 8
  }

  pub fn version_patch(&self) -> uint {
    (self.version_num as uint & 0xFF)
  }

  pub fn host<'a>(&'a self) -> &'a str {
    as_str(self.host)
  }

  pub fn is_ipv6_enabled(&self) -> bool {
    (self.features & CURL_VERSION_IPV6) == CURL_VERSION_IPV6
  }

  pub fn is_kerbos4_enabled(&self) -> bool {
    (self.features & CURL_VERSION_KERBEROS4) == CURL_VERSION_KERBEROS4
  }

  pub fn is_ssl_enabled(&self) -> bool {
    (self.features & CURL_VERSION_SSL) == CURL_VERSION_SSL
  }

  pub fn is_libz_enabled(&self) -> bool {
    (self.features & CURL_VERSION_LIBZ) == CURL_VERSION_LIBZ
  }

  pub fn is_ntlm_enabled(&self) -> bool {
    (self.features & CURL_VERSION_NTLM) == CURL_VERSION_NTLM
  }

  pub fn is_gss_negotiate_enabled(&self) -> bool {
    (self.features & CURL_VERSION_GSSNEGOTIATE) == CURL_VERSION_GSSNEGOTIATE
  }

  pub fn is_debug_enabled(&self) -> bool {
    (self.features & CURL_VERSION_DEBUG) == CURL_VERSION_DEBUG
  }

  pub fn is_async_dns_enabled(&self) -> bool {
    (self.features & CURL_VERSION_ASYNCHDNS) == CURL_VERSION_ASYNCHDNS
  }

  pub fn is_spengo_enabled(&self) -> bool {
    (self.features & CURL_VERSION_SPNEGO) == CURL_VERSION_SPNEGO
  }

  pub fn is_large_file_enabled(&self) -> bool {
    (self.features & CURL_VERSION_LARGEFILE) == CURL_VERSION_LARGEFILE
  }

  pub fn is_idn_enabled(&self) -> bool {
    (self.features & CURL_VERSION_IDN) == CURL_VERSION_IDN
  }

  pub fn is_sspi_enabled(&self) -> bool {
    (self.features & CURL_VERSION_SSPI) == CURL_VERSION_SSPI
  }

  pub fn is_conv_enabled(&self) -> bool {
    (self.features & CURL_VERSION_CONV) == CURL_VERSION_CONV
  }

  pub fn is_curl_debug_enabled(&self) -> bool {
    (self.features & CURL_VERSION_CURLDEBUG) == CURL_VERSION_CURLDEBUG
  }

  pub fn is_tls_auth_srp_enabled(&self) -> bool {
    (self.features & CURL_VERSION_TLSAUTH_SRP) == CURL_VERSION_TLSAUTH_SRP
  }

  pub fn is_ntlm_wb_enabled(&self) -> bool {
    (self.features & CURL_VERSION_NTLM_WB) == CURL_VERSION_NTLM_WB
  }

  pub fn is_http2_enabled(&self) -> bool {
    (self.features & CURL_VERSION_HTTP2) == CURL_VERSION_HTTP2
  }

  pub fn ssl_version<'a>(&'a self) -> &'a str {
    as_str(self.ssl_version)
  }

  pub fn libz_version<'a>(&'a self) -> &'a str {
    as_str(self.libz_version)
  }

  pub fn protocols<'a>(&'a self) -> Protocols<'a> {
    Protocols { curr: self.protocols }
  }
}

#[deriving(Clone)]
pub struct Protocols<'a> {
  curr: **c_char
}

impl<'a> Iterator<&'a str> for Protocols<'a> {
  fn next(&mut self) -> Option<&'a str> {
    unsafe {
      let proto = *self.curr;

      if proto == ptr::null() {
        return None;
      }

      self.curr = self.curr.offset(1);
      Some(as_str(proto))
    }
  }
}

impl<'a> fmt::Show for Protocols<'a> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
    let mut i = self.clone();

    try!(write!(fmt, "["));

    match i.next() {
      Some(proto) => try!(write!(fmt, "{}", proto)),
      None => return write!(fmt, "]")
    }

    for proto in i {
      try!(write!(fmt, ", {}", proto));
    }

    write!(fmt, "]")
  }
}

fn as_str<'a>(p: *c_char) -> &'a str {
  unsafe {
    let v = CString::new(p, false);
    mem::transmute(v.as_str().unwrap())
  }
}

#[link(name = "curl")]
extern {
  fn curl_version() -> *c_char;
  fn curl_version_info(t: CURLversion) -> &'static curl_version_info_data;
}

pub fn version_info() -> &'static Version {
  unsafe { curl_version_info(CURL_VERSION_NOW) }
}

pub fn version() -> &'static str {
  unsafe {
    let v = CString::new(curl_version(), false);
    mem::transmute(v.as_str().unwrap())
  }
}
