use std::ffi::CStr;
use std::error;
use std::fmt;
use std::str;

use curl_ffi as ffi;
/* please call curl_multi_perform() or curl_multi_socket*() soon */

pub use curl_ffi::CURLMcode::CURLM_CALL_MULTI_PERFORM as OK;
/* the passed-in handle is not a valid CURLM handle */

pub use curl_ffi::CURLMcode::CURLM_BAD_HANDLE as BAD_HANDLE;
/* an easy handle was not good/valid */
pub use curl_ffi::CURLMcode::CURLM_BAD_EASY_HANDLE as EASY_HANDLE;
/* if you ever get this, you're in deep sh*t */
pub use curl_ffi::CURLMcode::CURLM_OUT_OF_MEMORY as OUT_OF_MEMORY;
/* this is a libcurl bug */
pub use curl_ffi::CURLMcode::CURLM_INTERNAL_ERROR as INTERNAL_ERROR;
/* the passed in socket argument did not match */
pub use curl_ffi::CURLMcode::CURLM_BAD_SOCKET as BAD_SOCKET;
/* curl_multi_setopt() with unsupported option */
pub use curl_ffi::CURLMcode::CURLM_UNKNOWN_OPTION as UNKNOWN_OPTION;
/* an easy handle already added to a multi handle was
                       attempted to get added - again */
pub use curl_ffi::CURLMcode::CURLM_ADDED_ALREADY as ADDED_ALREADY;

#[derive(Copy, Clone)]
pub struct ErrCodeM(pub ffi::CURLMcode);

impl ErrCodeM {
    pub fn is_success(self) -> bool {
       self.code() as i32 == OK as i32
    }

    pub fn code(self) -> ffi::CURLMcode { let ErrCodeM(c) = self; c }
}

impl fmt::Debug for ErrCodeM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ErrCodeM {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe {
            CStr::from_ptr(ffi::curl_multi_strerror(self.code())).to_bytes()
        };

        match str::from_utf8(s) {
            Ok(s) => write!(fmt, "{}", s),
            Err(err) => write!(fmt, "{}", err)
        }
    }
}

impl error::Error for ErrCodeM {
    fn description(&self) -> &str {
        let code = self.code();
        let s = unsafe {
            CStr::from_ptr(ffi::curl_multi_strerror(code) as *const _).to_bytes()
        };
        str::from_utf8(s).unwrap()
    }
}
