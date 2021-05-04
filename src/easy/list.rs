use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr;

use crate::Error;
use curl_sys;

/// A linked list of a strings
pub struct List {
    raw: *mut curl_sys::curl_slist,
}

/// An iterator over `List`
#[derive(Clone)]
pub struct Iter<'a> {
    _me: &'a List,
    cur: *mut curl_sys::curl_slist,
}

pub fn raw(list: &List) -> *mut curl_sys::curl_slist {
    list.raw
}

pub unsafe fn from_raw(raw: *mut curl_sys::curl_slist) -> List {
    List { raw }
}

unsafe impl Send for List {}

impl List {
    /// Creates a new empty list of strings.
    pub fn new() -> List {
        List {
            raw: ptr::null_mut(),
        }
    }

    /// Appends some data into this list.
    pub fn append(&mut self, data: &str) -> Result<(), Error> {
        let data = CString::new(data)?;
        unsafe {
            let raw = curl_sys::curl_slist_append(self.raw, data.as_ptr());
            assert!(!raw.is_null());
            self.raw = raw;
            Ok(())
        }
    }

    /// Returns an iterator over the nodes in this list.
    pub fn iter(&self) -> Iter {
        Iter {
            _me: self,
            cur: self.raw,
        }
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.iter().map(String::from_utf8_lossy))
            .finish()
    }
}

impl<'a> IntoIterator for &'a List {
    type IntoIter = Iter<'a>;
    type Item = &'a [u8];

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl Drop for List {
    fn drop(&mut self) {
        unsafe { curl_sys::curl_slist_free_all(self.raw) }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.cur.is_null() {
            return None;
        }

        unsafe {
            let ret = Some(CStr::from_ptr((*self.cur).data).to_bytes());
            self.cur = (*self.cur).next;
            ret
        }
    }
}

impl<'a> fmt::Debug for Iter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.clone().map(String::from_utf8_lossy))
            .finish()
    }
}
