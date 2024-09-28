use crate::easy::Easy2;
use crate::error::Error;
use curl_sys::{
    curl_mime, curl_mime_addpart, curl_mime_data, curl_mime_filename, curl_mime_free,
    curl_mime_init, curl_mime_name, curl_mime_type, curl_mimepart, CURLcode, CURL, CURLE_OK,
};
use std::ffi::CString;
use std::marker::PhantomData;

#[derive(Debug)]
pub(super) struct MimeHandle(pub *mut curl_mime);

impl MimeHandle {
    fn new(easy: *mut CURL) -> Self {
        let handle = unsafe { curl_mime_init(easy) };
        assert!(!handle.is_null());

        Self(handle)
    }
}

impl Drop for MimeHandle {
    fn drop(&mut self) {
        unsafe { curl_mime_free(self.0) }
    }
}

#[derive(Debug)]
pub struct Mime<'e, E> {
    pub(super) handle: MimeHandle,
    easy: &'e mut Easy2<E>,
}

impl<'a, T> Mime<'a, T> {
    /// Create a mime handle
    pub(super) fn new(easy: &'a mut Easy2<T>) -> Self {
        let handle = MimeHandle::new(easy.raw());

        Self { handle, easy }
    }

    /// Finalize creation of a mime post.
    pub fn post(self) -> Result<(), Error> {
        // We give ownership on `MimeHandle` to `Easy2`. `Easy2` will keep record of this object
        // until it is safe to free (drop) it.
        self.easy.mimepost(self.handle)
    }

    /// Append a new empty part to a mime structure
    pub fn add_part(&mut self) -> MimePart<'a> {
        MimePart::new(self)
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
        let handle = unsafe { curl_mime_addpart(mime.handle.0) };
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

#[cfg(test)]
mod tests {
    use crate::easy::Easy;

    /// Trivial test which checks that objects can be used as planned.
    #[test]
    fn test_ownership() {
        let mut easy = Easy::new();
        let mut mime = easy.add_mime();

        for i in 1..5 {
            let name = format!("name{i}");
            let data = format!("data{i}");
            let fname = format!("fname{i}");

            mime.add_part()
                .set_data(name)
                .unwrap()
                .set_data(data)
                .unwrap()
                .set_filename(&fname)
                .unwrap()
                .set_content_type("plain/text")
                .unwrap();
        }

        mime.post().unwrap();
    }
}
