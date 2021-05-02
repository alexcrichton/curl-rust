use std::ffi::CString;
use std::fmt;
use std::path::Path;
use std::ptr;

use crate::easy::{list, List};
use crate::FormError;
use curl_sys;

/// Multipart/formdata for an HTTP POST request.
///
/// This structure is built up and then passed to the `Easy::httppost` method to
/// be sent off with a request.
pub struct Form {
    head: *mut curl_sys::curl_httppost,
    tail: *mut curl_sys::curl_httppost,
    headers: Vec<List>,
    buffers: Vec<Vec<u8>>,
    strings: Vec<CString>,
}

/// One part in a multipart upload, added to a `Form`.
pub struct Part<'form, 'data> {
    form: &'form mut Form,
    name: &'data str,
    array: Vec<curl_sys::curl_forms>,
    error: Option<FormError>,
}

pub fn raw(form: &Form) -> *mut curl_sys::curl_httppost {
    form.head
}

impl Form {
    /// Creates a new blank form ready for the addition of new data.
    pub fn new() -> Form {
        Form {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            headers: Vec::new(),
            buffers: Vec::new(),
            strings: Vec::new(),
        }
    }

    /// Prepares adding a new part to this `Form`
    ///
    /// Note that the part is not actually added to the form until the `add`
    /// method is called on `Part`, which may or may not fail.
    pub fn part<'a, 'data>(&'a mut self, name: &'data str) -> Part<'a, 'data> {
        Part {
            error: None,
            form: self,
            name,
            array: vec![curl_sys::curl_forms {
                option: curl_sys::CURLFORM_END,
                value: ptr::null_mut(),
            }],
        }
    }
}

impl fmt::Debug for Form {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: fill this out more
        f.debug_struct("Form").field("fields", &"...").finish()
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        unsafe {
            curl_sys::curl_formfree(self.head);
        }
    }
}

impl<'form, 'data> Part<'form, 'data> {
    /// A pointer to the contents of this part, the actual data to send away.
    pub fn contents(&mut self, contents: &'data [u8]) -> &mut Self {
        let pos = self.array.len() - 1;

        // curl has an oddity where if the length if 0 it will call strlen
        // on the value.  This means that if someone wants to add empty form
        // contents we need to make sure the buffer contains a null byte.
        let ptr = if contents.is_empty() {
            b"\x00"
        } else {
            contents
        }
        .as_ptr();

        self.array.insert(
            pos,
            curl_sys::curl_forms {
                option: curl_sys::CURLFORM_COPYCONTENTS,
                value: ptr as *mut _,
            },
        );
        self.array.insert(
            pos + 1,
            curl_sys::curl_forms {
                option: curl_sys::CURLFORM_CONTENTSLENGTH,
                value: contents.len() as *mut _,
            },
        );
        self
    }

    /// Causes this file to be read and its contents used as data in this part
    ///
    /// This part does not automatically become a file upload part simply
    /// because its data was read from a file.
    ///
    /// # Errors
    ///
    /// If the filename has any internal nul bytes or if on Windows it does not
    /// contain a unicode filename then the `add` function will eventually
    /// return an error.
    pub fn file_content<P>(&mut self, file: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self._file_content(file.as_ref())
    }

    fn _file_content(&mut self, file: &Path) -> &mut Self {
        if let Some(bytes) = self.path2cstr(file) {
            let pos = self.array.len() - 1;
            self.array.insert(
                pos,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_FILECONTENT,
                    value: bytes.as_ptr() as *mut _,
                },
            );
            self.form.strings.push(bytes);
        }
        self
    }

    /// Makes this part a file upload part of the given file.
    ///
    /// Sets the filename field to the basename of the provided file name, and
    /// it reads the contents of the file and passes them as data and sets the
    /// content type if the given file matches one of the internally known file
    /// extensions.
    ///
    /// The given upload file must exist entirely on the filesystem before the
    /// upload is started because libcurl needs to read the size of it
    /// beforehand.
    ///
    /// Multiple files can be uploaded by calling this method multiple times and
    /// content types can also be configured for each file (by calling that
    /// next).
    ///
    /// # Errors
    ///
    /// If the filename has any internal nul bytes or if on Windows it does not
    /// contain a unicode filename then this function will cause `add` to return
    /// an error when called.
    pub fn file<P: ?Sized>(&mut self, file: &'data P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self._file(file.as_ref())
    }

    fn _file(&mut self, file: &'data Path) -> &mut Self {
        if let Some(bytes) = self.path2cstr(file) {
            let pos = self.array.len() - 1;
            self.array.insert(
                pos,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_FILE,
                    value: bytes.as_ptr() as *mut _,
                },
            );
            self.form.strings.push(bytes);
        }
        self
    }

    /// Used in combination with `Part::file`, provides the content-type for
    /// this part, possibly instead of choosing an internal one.
    ///
    /// # Panics
    ///
    /// This function will panic if `content_type` contains an internal nul
    /// byte.
    pub fn content_type(&mut self, content_type: &'data str) -> &mut Self {
        if let Some(bytes) = self.bytes2cstr(content_type.as_bytes()) {
            let pos = self.array.len() - 1;
            self.array.insert(
                pos,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_CONTENTTYPE,
                    value: bytes.as_ptr() as *mut _,
                },
            );
            self.form.strings.push(bytes);
        }
        self
    }

    /// Used in combination with `Part::file`, provides the filename for
    /// this part instead of the actual one.
    ///
    /// # Errors
    ///
    /// If `name` contains an internal nul byte, or if on Windows the path is
    /// not valid unicode then this function will return an error when `add` is
    /// called.
    pub fn filename<P: ?Sized>(&mut self, name: &'data P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self._filename(name.as_ref())
    }

    fn _filename(&mut self, name: &'data Path) -> &mut Self {
        if let Some(bytes) = self.path2cstr(name) {
            let pos = self.array.len() - 1;
            self.array.insert(
                pos,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_FILENAME,
                    value: bytes.as_ptr() as *mut _,
                },
            );
            self.form.strings.push(bytes);
        }
        self
    }

    /// This is used to provide a custom file upload part without using the
    /// `file` method above.
    ///
    /// The first parameter is for the filename field and the second is the
    /// in-memory contents.
    ///
    /// # Errors
    ///
    /// If `name` contains an internal nul byte, or if on Windows the path is
    /// not valid unicode then this function will return an error when `add` is
    /// called.
    pub fn buffer<P: ?Sized>(&mut self, name: &'data P, data: Vec<u8>) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self._buffer(name.as_ref(), data)
    }

    fn _buffer(&mut self, name: &'data Path, mut data: Vec<u8>) -> &mut Self {
        if let Some(bytes) = self.path2cstr(name) {
            // If `CURLFORM_BUFFERLENGTH` is set to `0`, libcurl will instead do a strlen() on the
            // contents to figure out the size so we need to make sure the buffer is actually
            // zero terminated.
            let length = data.len();
            if length == 0 {
                data.push(0);
            }

            let pos = self.array.len() - 1;
            self.array.insert(
                pos,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_BUFFER,
                    value: bytes.as_ptr() as *mut _,
                },
            );
            self.form.strings.push(bytes);
            self.array.insert(
                pos + 1,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_BUFFERPTR,
                    value: data.as_ptr() as *mut _,
                },
            );
            self.array.insert(
                pos + 2,
                curl_sys::curl_forms {
                    option: curl_sys::CURLFORM_BUFFERLENGTH,
                    value: length as *mut _,
                },
            );
            self.form.buffers.push(data);
        }
        self
    }

    /// Specifies extra headers for the form POST section.
    ///
    /// Appends the list of headers to those libcurl automatically generates.
    pub fn content_header(&mut self, headers: List) -> &mut Self {
        let pos = self.array.len() - 1;
        self.array.insert(
            pos,
            curl_sys::curl_forms {
                option: curl_sys::CURLFORM_CONTENTHEADER,
                value: list::raw(&headers) as *mut _,
            },
        );
        self.form.headers.push(headers);
        self
    }

    /// Attempts to add this part to the `Form` that it was created from.
    ///
    /// If any error happens while adding, that error is returned, otherwise
    /// `Ok(())` is returned.
    pub fn add(&mut self) -> Result<(), FormError> {
        if let Some(err) = self.error.clone() {
            return Err(err);
        }
        let rc = unsafe {
            curl_sys::curl_formadd(
                &mut self.form.head,
                &mut self.form.tail,
                curl_sys::CURLFORM_COPYNAME,
                self.name.as_ptr(),
                curl_sys::CURLFORM_NAMELENGTH,
                self.name.len(),
                curl_sys::CURLFORM_ARRAY,
                self.array.as_ptr(),
                curl_sys::CURLFORM_END,
            )
        };
        if rc == curl_sys::CURL_FORMADD_OK {
            Ok(())
        } else {
            Err(FormError::new(rc))
        }
    }

    #[cfg(unix)]
    fn path2cstr(&mut self, p: &Path) -> Option<CString> {
        use std::os::unix::prelude::*;
        self.bytes2cstr(p.as_os_str().as_bytes())
    }

    #[cfg(windows)]
    fn path2cstr(&mut self, p: &Path) -> Option<CString> {
        match p.to_str() {
            Some(bytes) => self.bytes2cstr(bytes.as_bytes()),
            None if self.error.is_none() => {
                // TODO: better error code
                self.error = Some(FormError::new(curl_sys::CURL_FORMADD_INCOMPLETE));
                None
            }
            None => None,
        }
    }

    fn bytes2cstr(&mut self, bytes: &[u8]) -> Option<CString> {
        match CString::new(bytes) {
            Ok(c) => Some(c),
            Err(..) if self.error.is_none() => {
                // TODO: better error code
                self.error = Some(FormError::new(curl_sys::CURL_FORMADD_INCOMPLETE));
                None
            }
            Err(..) => None,
        }
    }
}

impl<'form, 'data> fmt::Debug for Part<'form, 'data> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: fill this out more
        f.debug_struct("Part")
            .field("name", &self.name)
            .field("form", &self.form)
            .finish()
    }
}
