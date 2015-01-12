#![allow(dead_code)]

use std::ptr;
use libc::c_void;
use super::opt::OptVal;

use curl_ffi as ffi;

pub struct List {
    len: usize,
    head: *mut ffi::curl_slist,
}

impl List {
    pub fn new() -> List {
        List {
            len: 0,
            head: ptr::null_mut()
        }
    }

    pub fn push_bytes(&mut self, val: &[u8]) {
        assert!(val[val.len() - 1] == 0);
        self.len += 1;
        self.head = unsafe { ffi::curl_slist_append(self.head, val.as_ptr()) };
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for List {
    fn drop(&mut self) {
        if !self.is_empty() {
            unsafe { ffi::curl_slist_free_all(self.head) }
        }
    }
}

impl<'a> OptVal for &'a List {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self.head as *const c_void)
    }
}
