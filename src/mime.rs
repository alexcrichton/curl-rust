//! MIME handling in libcurl.

use std::{
    ffi::{c_void, CString},
    io::SeekFrom,
    slice,
};

use crate::{
    easy::{list::raw as list_raw, Easy2, List, ReadError},
    panic, Error,
};

mod handler;

pub use handler::PartDataHandler;
use libc::{c_char, c_int, size_t};

#[derive(Debug)]
pub(crate) struct MimeHandle(pub(crate) *mut curl_sys::curl_mime);

/// A MIME handle that holds MIME parts.
#[must_use = "Mime is not attached to the Easy handle until you call `post()` on it"]
#[derive(Debug)]
pub struct Mime<'h, H> {
    handle: MimeHandle,
    easy: &'h mut Easy2<H>,
}

/// A MIME part associated with a MIME handle.
#[derive(Debug)]
pub struct MimePart<'m, 'h, H> {
    raw: *mut curl_sys::curl_mimepart,
    mime: &'m Mime<'h, H>,
}

unsafe impl<H> Send for Mime<'_, H> {}

impl<'h, H> Mime<'h, H> {
    pub(crate) fn new(easy: &'h mut Easy2<H>) -> Self {
        let raw = unsafe { curl_sys::curl_mime_init(easy.raw()) };
        assert!(!raw.is_null());
        Self {
            handle: MimeHandle(raw),
            easy,
        }
    }

    /// Creates a new MIME part associated to this MIME handle.
    pub fn add_part<'m>(&'m self) -> MimePart<'m, 'h, H> {
        MimePart::new(self)
    }

    /// Returns the raw MIME handle pointer.
    pub fn raw(&self) -> *mut curl_sys::curl_mime {
        self.handle.0
    }

    /// Pass the MIME handle to the originating Easy handle to post an HTTP form.
    ///
    /// This option corresponds to `CURLOPT_MIMEPOST`.
    pub fn post(self) -> Result<(), Error> {
        self.easy.set_mime(self.handle)?;
        Ok(())
    }
}

impl<'m, 'h, H> MimePart<'m, 'h, H> {
    fn new(mime: &'m Mime<'h, H>) -> Self {
        let raw = unsafe { curl_sys::curl_mime_addpart(mime.handle.0) };
        assert!(!raw.is_null());
        Self { raw, mime }
    }

    /// Returns the raw MIME part pointer.
    pub fn raw(&self) -> *mut curl_sys::curl_mimepart {
        self.raw
    }

    /// Sets the data of the content of this MIME part.
    pub fn data(&mut self, data: &[u8]) -> Result<(), Error> {
        let code =
            unsafe { curl_sys::curl_mime_data(self.raw, data.as_ptr() as *const _, data.len()) };
        self.mime.easy.cvt(code)?;
        Ok(())
    }

    /// Sets the name of this MIME part.
    pub fn name(&mut self, name: &str) -> Result<(), Error> {
        let name = CString::new(name)?;
        let code = unsafe { curl_sys::curl_mime_name(self.raw, name.as_ptr()) };
        self.mime.easy.cvt(code)?;
        Ok(())
    }

    /// Sets the filename of this MIME part.
    pub fn filename(&mut self, filename: &str) -> Result<(), Error> {
        let filename = CString::new(filename)?;
        let code = unsafe { curl_sys::curl_mime_filename(self.raw, filename.as_ptr()) };
        self.mime.easy.cvt(code)?;
        Ok(())
    }

    /// Sets the content type of this MIME part.
    pub fn content_type(&mut self, content_type: &str) -> Result<(), Error> {
        let content_type = CString::new(content_type)?;
        let code = unsafe { curl_sys::curl_mime_type(self.raw, content_type.as_ptr()) };
        self.mime.easy.cvt(code)?;
        Ok(())
    }

    /// Sets the list of headers of this MIME part.
    pub fn headers(&mut self, header_list: List) -> Result<(), Error> {
        let header_list = std::mem::ManuallyDrop::new(header_list);
        let code = unsafe { curl_sys::curl_mime_headers(self.raw, list_raw(&header_list), 1) };
        self.mime.easy.cvt(code)?;
        Ok(())
    }

    /// Sets the handler that provides content data for this MIME part.
    pub fn data_handler<P: PartDataHandler + Send + 'static>(
        &mut self,
        size: usize,
        handler: P,
    ) -> Result<(), Error> {
        let mut inner = Box::new(handler);
        let code = unsafe {
            curl_sys::curl_mime_data_cb(
                self.raw,
                size as curl_sys::curl_off_t,
                Some(read_cb::<P>),
                Some(seek_cb::<P>),
                Some(free_handler::<P>),
                &mut *inner as *mut _ as *mut c_void,
            )
        };
        self.mime.easy.cvt(code)?;
        Box::leak(inner);
        Ok(())
    }
}

impl Drop for MimeHandle {
    fn drop(&mut self) {
        unsafe { curl_sys::curl_mime_free(self.0) }
    }
}

extern "C" fn read_cb<P: PartDataHandler + Send + 'static>(
    ptr: *mut c_char,
    size: size_t,
    nmemb: size_t,
    data: *mut c_void,
) -> size_t {
    panic::catch(|| unsafe {
        let input = slice::from_raw_parts_mut(ptr as *mut u8, size * nmemb);
        match (*(data as *mut P)).read(input) {
            Ok(s) => s,
            Err(ReadError::Pause) => curl_sys::CURL_READFUNC_PAUSE,
            Err(ReadError::Abort) => curl_sys::CURL_READFUNC_ABORT,
        }
    })
    .unwrap_or(!0)
}

extern "C" fn seek_cb<P: PartDataHandler + Send + 'static>(
    data: *mut c_void,
    offset: curl_sys::curl_off_t,
    origin: c_int,
) -> c_int {
    panic::catch(|| unsafe {
        let from = if origin == libc::SEEK_SET {
            SeekFrom::Start(offset as u64)
        } else {
            panic!("unknown origin from libcurl: {}", origin);
        };
        (*(data as *mut P)).seek(from) as c_int
    })
    .unwrap_or(!0)
}

extern "C" fn free_handler<P: PartDataHandler + Send + 'static>(data: *mut c_void) {
    panic::catch(|| unsafe {
        let _ = Box::from_raw(data as *mut P);
    })
    .unwrap_or(());
}
