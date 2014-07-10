#![allow(non_camel_case_types)]

use std::{ptr, raw, mem};
use std::c_str::CString;

use libc::{c_void, c_char};
use super::opt::OptVal;

#[repr(C)]
struct curl_slist {
    data: *const c_char,
    next: *mut curl_slist,
}

#[link(name = "curl")]
extern {
  fn curl_slist_append(list: *mut curl_slist, val: *const u8) -> *mut curl_slist;
  fn curl_slist_free_all(list: *mut curl_slist);
}

pub struct List {
  len: uint,
  head: *mut curl_slist,
}

pub struct Items<'a> {
  _list: &'a List,
  cur: *mut curl_slist,
}

impl List {
  pub fn new() -> List {
    List {
      len: 0,
      head: ptr::mut_null()
    }
  }

  pub fn push_bytes(&mut self, val: &[u8]) {
    assert!(val[val.len() - 1] == 0);
    self.len += 1;
    self.head = unsafe { curl_slist_append(self.head, val.as_ptr()) };
  }

  pub fn iter<'a>(&'a self) -> Items<'a> {
    Items { _list: self, cur: self.head }
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
  fn with_c_repr(self, f: |*const c_void|) {
    f(self.head as *const c_void)
  }
}

impl<'a> Iterator<&'a [u8]> for Items<'a> {
  fn next(&mut self) -> Option<&'a [u8]> {
    if self.cur as uint == 0 {
      None
    } else {
      unsafe {
        let ptr = (*self.cur).data;
        let len = CString::new(ptr, false).len();
        self.cur = (*self.cur).next;
        Some(mem::transmute(raw::Slice { data: ptr, len: len }))
      }
    }
  }
}

#[cfg(test)]
mod tests {
    use super::List;

    #[test]
    fn simple_iter() {
        let mut l = List::new();
        assert!(l.iter().count() == 0);
        l.push_bytes(b"foo\0");
        l.push_bytes(b"bar\0");

        let mut i = l.iter();
        assert_eq!(i.next(), Some(b"foo"));
        assert_eq!(i.next(), Some(b"bar"));
        assert_eq!(i.next(), None);
    }

    #[test]
    #[should_fail]
    fn needs_nul() {
        let mut l = List::new();
        l.push_bytes(b"foo");
    }
}
