#![crate_id = "curl"]
#![feature(macro_rules)]

extern crate collections;
extern crate libc;

use std::io;
use ffi::{easy,opt};
use ffi::err::ErrCode;
use handle::Handle;
use body::Body;

pub use response::{Headers,Response};

mod body;
mod ffi;
mod handle;
mod header;
mod response;

#[cfg(test)]
mod test;

pub fn request<'a>() -> Request<'a> {
  Request::new()
}

pub fn get(uri: &str) -> Result<Response, ErrCode> {
  request()
    .method(Get)
    .uri(uri)
    .execute()
}

pub fn post<R: Reader>(uri: &str, body: &mut R) -> Result<Response, ErrCode> {
  request()
    .method(Post)
    .uri(uri)
    // .header("Transfer-Encoding", "chunked")
    // .header("Content-Type", "text/plain")
    .header("Expect", "")
    .body(body)
    .execute()
}

pub enum Method {
  Options,
  Get,
  Head,
  Post,
  Put,
  Delete,
  Trace,
  Connect
}

pub struct Request<'a> {
  err: Option<ErrCode>,
  handle: Handle,
  headers: ffi::List,
  body: Option<Body<'a>>
}

impl<'a> Request<'a> {
  fn new() -> Request<'a> {
    Request {
      err: None,
      handle: Handle::new(),
      headers: ffi::List::new(),
      body: None
    }
  }

  pub fn method(mut self, method: Method) -> Request<'a> {
    macro_rules! set_method(
      ($val:expr) => ({
        match self.handle.setopt($val, 1) {
          Ok(_) => {}
          Err(e) => self.err = Some(e)
        }
        });
    )

    // TODO: track errors
    match method {
      Get => set_method!(opt::HTTPGET),
      Post => set_method!(opt::POST),
      _ => { unimplemented!() }
    }

    self
  }

  pub fn uri(mut self, uri: &str) -> Request<'a> {
    match self.handle.setopt(opt::URL, uri) {
      Ok(_) => {}
      Err(e) => self.err = Some(e)
    }

    self
  }

  pub fn header(mut self, name: &str, val: &str) -> Request<'a> {
    self.append_header(name, val);
    self
  }

  pub fn headers<'a, I: Iterator<(&'a str, &'a str)>>(mut self, mut hdrs: I) -> Request<'a> {
    for (name, val) in hdrs {
      self.append_header(name, val);
    }

    self
  }

  fn append_header(&mut self, name: &str, val: &str) {
    let mut c_str = Vec::with_capacity(name.len() + val.len() + 2);
    c_str.push_all(name.as_bytes());
    c_str.push(':' as u8);
    c_str.push_all(val.as_bytes());
    c_str.push(0);

    self.headers.push_bytes(c_str.as_slice());
  }

  pub fn body<R: io::Reader>(mut self, r: &'a mut R) -> Request<'a> {
    self.body = Some(Body::new(r as &mut io::Reader));
    self
  }

  pub fn execute(mut self) -> Result<Response, ErrCode> {
    match self.err {
      Some(e) => return Err(e),
      None => {}
    }

    if !self.headers.is_empty() {
      try!(self.handle.setopt(opt::HTTPHEADER, &self.headers));
    }

    self.handle.perform(self.body.as_ref())
  }
}
