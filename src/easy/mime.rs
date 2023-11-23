use crate::easy::Easy2;
use crate::error::Error;
use curl_sys::{
    curl_mime_addpart, curl_mime_data, curl_mime_filename, curl_mime_free, curl_mime_init,
    curl_mime_name, curl_mime_type, curl_mimepart, CURLcode, CURLE_OK,
};
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::null_mut;

#[derive(Debug)]
pub struct Mime<'e, E> {
    handle: *mut curl_sys::curl_mime,
    easy: &'e mut Easy2<E>,
}

impl<'a, T> Mime<'a, T> {
    /// Create a mime handle
    pub(crate) fn new(easy: &'a mut Easy2<T>) -> Self {
        let handle = unsafe { curl_mime_init(easy.raw()) };
        assert!(!handle.is_null());

        Self { handle, easy }
    }

    /// Finalize creation of a mime post.
    pub fn post(mut self) -> Result<(), Error> {
        // once giving the mime handle to `Easy2` it is now their responsibility to free the handle.
        // so we need to make sure `Drop` below won't try to free it.
        let mime_handle = self.handle;
        self.handle = null_mut();
        self.easy.mimepost(mime_handle)
    }

    /// Append a new empty part to a mime structure
    pub fn add_part(&mut self) -> MimePart<'a> {
        MimePart::new(self)
    }
}

impl<E> Drop for Mime<'_, E> {
    fn drop(&mut self) {
        // we only need to free mime handles which hadn't been given to the ownership of `Easy2`.
        if !self.handle.is_null() {
            unsafe { curl_mime_free(self.handle) }
        }
    }
}

#[derive(Debug)]
pub struct MimePart<'a> {
    handle: *mut curl_mimepart,
    // attach to the lifetime of our [Mime] handle, but without taking ownership
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> MimePart<'a> {
    fn new<E>(mime: &mut Mime<E>) -> Self {
        let handle = unsafe { curl_mime_addpart(mime.handle) };
        assert!(!handle.is_null());

        Self {
            handle,
            _lifetime: Default::default(),
        }
    }

    /// Set a mime part's body data
    pub fn set_data(self, data: impl AsRef<[u8]>) -> Result<Self, Error> {
        let data = data.as_ref();
        let code = unsafe { curl_mime_data(self.handle, data.as_ptr() as *const _, data.len()) };
        code_ok(code).map(|_| self)
    }

    /// Set a mime part's name
    ///
    /// # Panics
    /// If `name` contains nul bytes, panic will occur.
    pub fn set_name(self, name: &str) -> Result<Self, Error> {
        let data = CString::new(name).unwrap();
        let code = unsafe { curl_mime_name(self.handle, data.as_ptr()) };
        code_ok(code).map(|_| self)
    }

    /// Set a mime part's remote file name
    ///
    /// # Panics
    /// If `filename` contains nul bytes, panic will occur.
    pub fn set_filename(self, filename: &str) -> Result<Self, Error> {
        let data = CString::new(filename).unwrap();
        let code = unsafe { curl_mime_filename(self.handle, data.as_ptr()) };
        code_ok(code).map(|_| self)
    }

    /// Set a mime part's content type
    ///
    /// # Panics
    /// If `content_type` contains nul bytes, panic will occur.
    pub fn set_content_type(self, content_type: &str) -> Result<Self, Error> {
        let data = CString::new(content_type).unwrap();
        let code = unsafe { curl_mime_type(self.handle, data.as_ptr()) };
        code_ok(code).map(|_| self)
    }
}

fn code_ok(code: CURLcode) -> Result<(), Error> {
    if code == CURLE_OK {
        Ok(())
    } else {
        Err(Error::new(code))
    }
}
