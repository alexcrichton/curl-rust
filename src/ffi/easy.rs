use std::sync::{Once, ONCE_INIT};
use std::c_vec::CVec;
use std::{io,mem};
use std::collections::HashMap;
use libc::{c_void,c_int,c_long,c_double,size_t};
use super::{consts,err,info,opt};
use super::err::ErrCode;
use http::body::Body;
use http::{header,Response};

type CURL = c_void;
pub type ProgressCb<'a> = |uint, uint, uint, uint|:'a -> ();

#[link(name = "curl")]
extern {
  pub fn curl_easy_init() -> *mut CURL;
  pub fn curl_easy_setopt(curl: *mut CURL, option: opt::Opt, ...) -> ErrCode;
  pub fn curl_easy_perform(curl: *mut CURL) -> ErrCode;
  pub fn curl_easy_cleanup(curl: *mut CURL);
  pub fn curl_easy_getinfo(curl: *const CURL, info: info::Key, ...) -> ErrCode;
  pub fn curl_global_cleanup();
}

pub struct Easy {
  curl: *mut CURL
}

impl Easy {
  pub fn new() -> Easy {
    // Ensure that curl is globally initialized
    global_init();

    Easy {
      curl: unsafe {
        let p = curl_easy_init();
        curl_easy_setopt(p, opt::NOPROGRESS, 0u);
        p
      }
    }
  }

  #[inline]
  pub fn setopt<T: opt::OptVal>(&mut self, option: opt::Opt, val: T) -> Result<(), err::ErrCode> {
    // TODO: Prevent setting callback related options
    let mut res = err::OK;

    unsafe {
      val.with_c_repr(|repr| {
        res = curl_easy_setopt(self.curl, option, repr);
      })
    }

    if res.is_success() { Ok(()) } else { Err(res) }
  }

  #[inline]
  pub fn perform(&mut self, body: Option<&mut Body>, progress: Option<ProgressCb>) -> Result<Response, err::ErrCode> {
    let mut builder = ResponseBuilder::new();

    unsafe {
      let resp_p: uint = mem::transmute(&builder);
      let body_p: uint = match body {
        Some(b) => mem::transmute(b),
        None => 0
      };

      let progress_p: uint = match progress.as_ref() {
        Some(cb) => mem::transmute(cb),
        None => 0
      };

      debug!("setting read fn: {}", body_p != 0);

      // Set callback options
      curl_easy_setopt(self.curl, opt::READFUNCTION, curl_read_fn);
      curl_easy_setopt(self.curl, opt::READDATA, body_p);

      curl_easy_setopt(self.curl, opt::WRITEFUNCTION, curl_write_fn);
      curl_easy_setopt(self.curl, opt::WRITEDATA, resp_p);

      curl_easy_setopt(self.curl, opt::HEADERFUNCTION, curl_header_fn);
      curl_easy_setopt(self.curl, opt::HEADERDATA, resp_p);

      curl_easy_setopt(self.curl, opt::PROGRESSFUNCTION, curl_progress_fn);
      curl_easy_setopt(self.curl, opt::PROGRESSDATA, progress_p);
    }

    let err = unsafe { curl_easy_perform(self.curl) };

    // If the request failed, abort here
    if !err.is_success() {
      return Err(err);
    }

    // Try to get the response code
    builder.code = try!(self.get_response_code());

    Ok(builder.build())
  }

  pub fn get_response_code(&self) -> Result<uint, err::ErrCode> {
    Ok(try!(self.get_info_long(info::RESPONSE_CODE)) as uint)
  }
  
  pub fn get_total_time(&self) -> Result<uint, err::ErrCode> {
    Ok(try!(self.get_info_long(info::TOTAL_TIME)) as uint)
  }

  fn get_info_long(&self, key: info::Key) -> Result<c_long, err::ErrCode> {
    let v: c_long = 0;
    let res = unsafe { curl_easy_getinfo(self.curl as *const CURL, key, &v) };

    if !res.is_success() {
      return Err(res);
    }

    Ok(v)
  }
}

#[inline]
fn global_init() {
  // Schedule curl to be cleaned up after we're done with this whole process
  static mut INIT: Once = ONCE_INIT;
  unsafe {
    INIT.doit(|| ::std::rt::at_exit(proc() curl_global_cleanup()))
  }
}

impl Drop for Easy {
  fn drop(&mut self) {
    unsafe { curl_easy_cleanup(self.curl) }
  }
}

/*
 *
 * TODO: Move this into handle
 *
 */

struct ResponseBuilder {
  code: uint,
  hdrs: HashMap<String,Vec<String>>,
  body: Vec<u8>
}

impl ResponseBuilder {
  fn new() -> ResponseBuilder {
    ResponseBuilder {
      code: 0,
      hdrs: HashMap::new(),
      body: Vec::new()
    }
  }

  fn add_header(&mut self, name: &str, val: &str) {
    // TODO: Reduce allocations
    use std::ascii::OwnedStrAsciiExt;
    let name = name.to_string().into_ascii_lower();

    let inserted = match self.hdrs.find_mut(&name) {
      Some(vals) => {
        vals.push(val.to_string());
        true
      }
      None => false
    };

    if !inserted {
      self.hdrs.insert(name, vec!(val.to_string()));
    }
  }

  fn build(self) -> Response {
    let ResponseBuilder { code, hdrs, body } = self;
    Response::new(code, hdrs, body)
  }
}

/*
 *
 * ===== Callbacks =====
 */

pub extern "C" fn curl_read_fn(p: *mut u8, size: size_t, nmemb: size_t, body: *mut Body) -> size_t {
  if body.is_null() {
    return 0;
  }

  let mut dst = unsafe { CVec::new(p, (size * nmemb) as uint) };
  let body: &mut Body = unsafe { mem::transmute(body) };

  match body.read(dst.as_mut_slice()) {
    Ok(len) => len as size_t,
    Err(e) => {
      match e.kind {
        io::EndOfFile => 0 as size_t,
        _ => consts::CURL_READFUNC_ABORT as size_t
      }
    }
  }
}

pub extern "C" fn curl_write_fn(p: *mut u8, size: size_t, nmemb: size_t, resp: *mut ResponseBuilder) -> size_t {
  if !resp.is_null() {
    let builder: &mut ResponseBuilder = unsafe { mem::transmute(resp) };
    let chunk = unsafe { CVec::new(p, (size * nmemb) as uint) };
    builder.body.push_all(chunk.as_slice());
  }

  size * nmemb
}

pub extern "C" fn curl_header_fn(p: *mut u8, size: size_t, nmemb: size_t, resp: &mut ResponseBuilder) -> size_t {
  // TODO: Skip the first call (it seems to be the status line)

  let vec = unsafe { CVec::new(p, (size * nmemb) as uint) };

  match header::parse(vec.as_slice()) {
    Some((name, val)) => {
      resp.add_header(name, val);
    }
    None => {}
  }

  vec.len() as size_t
}

pub extern "C" fn curl_progress_fn(cb: *mut ProgressCb, dltotal: c_double, dlnow: c_double, ultotal: c_double, ulnow: c_double) -> c_int {
  #[inline]
  fn to_uint(v: c_double) -> uint {
    if v > 0.0 { v as uint } else { 0 }
  }

  if !cb.is_null() {
    let cb: &mut ProgressCb = unsafe { &mut *cb };
    (*cb)(to_uint(dltotal), to_uint(dlnow), to_uint(ultotal), to_uint(ulnow));
  }

  0
}
