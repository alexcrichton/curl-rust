use libc::{c_long,size_t};
use std::c_vec::CVec;
use std::collections::HashMap;
use std::{io,mem};
use super::ffi::easy::Easy;
use super::body::Body;
use super::ffi;
use super::ffi::{consts,easy,err,info,opt};
use {ErrCode,Response,header};

pub struct Handle {
  easy: Easy,
}

impl Handle {
  pub fn new() -> Handle {
    Handle {
      easy: Easy::new()
    }
  }

  pub fn get<'a, 'b>(&'a mut self, uri: &str) -> Request<'a, 'b> {
    Request::new(self, Get, None).uri(uri)
  }

  /*
  pub fn post<'a, 'b, R: Reader>(&'a mut self, &'b mut R) -> Request<'a, 'b> {
    unimplemented!();
  }
  */
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

pub struct Request<'a, 'b> {
  err: Option<ErrCode>,
  handle: &'a mut Handle,
  headers: ffi::List,
  body: Option<Body<'b>>
}

impl<'a, 'b> Request<'a, 'b> {
  fn new<'a, 'b>(handle: &'a mut Handle, method: Method, body: Option<Body<'b>>) -> Request<'a, 'b> {
    macro_rules! set_method(
      ($val:expr) => ({
        match handle.easy.setopt($val, 1) {
          Ok(_) => { None }
          Err(e) => Some(e)
        }
      });
    )

    // TODO: track errors
    let err = match method {
      Get => set_method!(opt::HTTPGET),
      Post => set_method!(opt::POST),
      _ => { unimplemented!() }
    };

    Request {
      err: err,
      handle: handle,
      headers: ffi::List::new(),
      body: body
    }
  }

  pub fn method(mut self, method: Method) -> Request<'a, 'b> {
    macro_rules! set_method(
      ($val:expr) => ({
        match self.handle.easy.setopt($val, 1) {
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

  pub fn uri(mut self, uri: &str) -> Request<'a, 'b> {
    match self.handle.easy.setopt(opt::URL, uri) {
      Ok(_) => {}
      Err(e) => self.err = Some(e)
    }

    self
  }

  pub fn header(mut self, name: &str, val: &str) -> Request<'a, 'b> {
    self.append_header(name, val);
    self
  }

  pub fn headers<'c, I: Iterator<(&'c str, &'c str)>>(mut self, mut hdrs: I) -> Request<'a, 'b> {
    for (name, val) in hdrs {
      self.append_header(name, val);
    }

    self
  }

  fn append_header(&mut self, name: &str, val: &str) {
    let mut c_str = Vec::with_capacity(name.len() + val.len() + 3);
    c_str.push_all(name.as_bytes());
    c_str.push(':' as u8);
    c_str.push(' ' as u8);
    c_str.push_all(val.as_bytes());
    c_str.push(0);

    self.headers.push_bytes(c_str.as_slice());
  }

  pub fn exec(mut self) -> Result<Response, ErrCode> {
    match self.err {
      Some(e) => return Err(e),
      None => {}
    }

    if !self.headers.is_empty() {
      try!(self.handle.easy.setopt(opt::HTTPHEADER, &self.headers));
    }

    self.handle.easy.perform(self.body.as_ref())
  }
}
