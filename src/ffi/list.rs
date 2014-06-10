use std::{mem,ptr};
use libc::{c_void};
use super::opt::OptVal;

type Node = c_void;

#[link(name = "curl")]
extern {
  fn curl_slist_append(list: *Node, val: *u8) -> *Node;
  fn curl_slist_free_all(list: *Node);
}

pub struct List {
  len: uint,
  head: *Node,
}

impl List {
  pub fn new() -> List {
    List {
      len: 0,
      head: ptr::null()
    }
  }

  pub fn push_str(&mut self, val: &str) {
    self.push_bytes(val.as_bytes());
  }

  pub fn push_bytes(&mut self, val: &[u8]) {
    self.len += 1;
    self.head = unsafe { curl_slist_append(self.head, val.as_ptr()) };
  }
}

impl Collection for List {
  fn len(&self) -> uint {
    self.len
  }

  fn is_empty(&self) -> bool {
    self.len == 0
  }
}

impl Drop for List {
  fn drop(&mut self) {
    if !self.is_empty() {
      unsafe { curl_slist_free_all(self.head) }
    }
  }
}

impl<'a> OptVal for &'a List {
  fn with_c_repr(self, f: |*c_void|) {
    f(self.head as *c_void)
  }
}
