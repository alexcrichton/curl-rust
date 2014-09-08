#![allow(non_camel_case_types)]

use std::ptr;
use libc::c_void;
use super::opt::OptVal;

type curl_slist = c_void;

extern {
    fn curl_slist_append(list: *mut curl_slist, val: *const u8) -> *mut curl_slist;
    fn curl_slist_free_all(list: *mut curl_slist);
}

pub struct List {
    len: uint,
    head: *mut curl_slist,
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
