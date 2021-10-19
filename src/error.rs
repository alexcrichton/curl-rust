use std::error;
use std::ffi::{self, CStr};
use std::fmt;
use std::io;
use std::str;

/// An error returned from various "easy" operations.
///
/// This structure wraps a `CURLcode`.
#[derive(Clone, PartialEq)]
pub struct Error {
    code: curl_sys::CURLcode,
    extra: Option<Box<str>>,
}

impl Error {
    /// Creates a new error from the underlying code returned by libcurl.
    pub fn new(code: curl_sys::CURLcode) -> Error {
        Error { code, extra: None }
    }

    /// Stores some extra information about this error inside this error.
    ///
    /// This is typically used with `take_error_buf` on the easy handles to
    /// couple the extra `CURLOPT_ERRORBUFFER` information with an `Error` being
    /// returned.
    pub fn set_extra(&mut self, extra: String) {
        self.extra = Some(extra.into());
    }

    /// Returns whether this error corresponds to CURLE_UNSUPPORTED_PROTOCOL.
    pub fn is_unsupported_protocol(&self) -> bool {
        self.code == curl_sys::CURLE_UNSUPPORTED_PROTOCOL
    }

    /// Returns whether this error corresponds to CURLE_FAILED_INIT.
    pub fn is_failed_init(&self) -> bool {
        self.code == curl_sys::CURLE_FAILED_INIT
    }

    /// Returns whether this error corresponds to CURLE_URL_MALFORMAT.
    pub fn is_url_malformed(&self) -> bool {
        self.code == curl_sys::CURLE_URL_MALFORMAT
    }

    // /// Returns whether this error corresponds to CURLE_NOT_BUILT_IN.
    // pub fn is_not_built_in(&self) -> bool {
    //     self.code == curl_sys::CURLE_NOT_BUILT_IN
    // }

    /// Returns whether this error corresponds to CURLE_COULDNT_RESOLVE_PROXY.
    pub fn is_couldnt_resolve_proxy(&self) -> bool {
        self.code == curl_sys::CURLE_COULDNT_RESOLVE_PROXY
    }

    /// Returns whether this error corresponds to CURLE_COULDNT_RESOLVE_HOST.
    pub fn is_couldnt_resolve_host(&self) -> bool {
        self.code == curl_sys::CURLE_COULDNT_RESOLVE_HOST
    }

    /// Returns whether this error corresponds to CURLE_COULDNT_CONNECT.
    pub fn is_couldnt_connect(&self) -> bool {
        self.code == curl_sys::CURLE_COULDNT_CONNECT
    }

    /// Returns whether this error corresponds to CURLE_REMOTE_ACCESS_DENIED.
    pub fn is_remote_access_denied(&self) -> bool {
        self.code == curl_sys::CURLE_REMOTE_ACCESS_DENIED
    }

    /// Returns whether this error corresponds to CURLE_PARTIAL_FILE.
    pub fn is_partial_file(&self) -> bool {
        self.code == curl_sys::CURLE_PARTIAL_FILE
    }

    /// Returns whether this error corresponds to CURLE_QUOTE_ERROR.
    pub fn is_quote_error(&self) -> bool {
        self.code == curl_sys::CURLE_QUOTE_ERROR
    }

    /// Returns whether this error corresponds to CURLE_HTTP_RETURNED_ERROR.
    pub fn is_http_returned_error(&self) -> bool {
        self.code == curl_sys::CURLE_HTTP_RETURNED_ERROR
    }

    /// Returns whether this error corresponds to CURLE_READ_ERROR.
    pub fn is_read_error(&self) -> bool {
        self.code == curl_sys::CURLE_READ_ERROR
    }

    /// Returns whether this error corresponds to CURLE_WRITE_ERROR.
    pub fn is_write_error(&self) -> bool {
        self.code == curl_sys::CURLE_WRITE_ERROR
    }

    /// Returns whether this error corresponds to CURLE_UPLOAD_FAILED.
    pub fn is_upload_failed(&self) -> bool {
        self.code == curl_sys::CURLE_UPLOAD_FAILED
    }

    /// Returns whether this error corresponds to CURLE_OUT_OF_MEMORY.
    pub fn is_out_of_memory(&self) -> bool {
        self.code == curl_sys::CURLE_OUT_OF_MEMORY
    }

    /// Returns whether this error corresponds to CURLE_OPERATION_TIMEDOUT.
    pub fn is_operation_timedout(&self) -> bool {
        self.code == curl_sys::CURLE_OPERATION_TIMEDOUT
    }

    /// Returns whether this error corresponds to CURLE_RANGE_ERROR.
    pub fn is_range_error(&self) -> bool {
        self.code == curl_sys::CURLE_RANGE_ERROR
    }

    /// Returns whether this error corresponds to CURLE_HTTP_POST_ERROR.
    pub fn is_http_post_error(&self) -> bool {
        self.code == curl_sys::CURLE_HTTP_POST_ERROR
    }

    /// Returns whether this error corresponds to CURLE_SSL_CONNECT_ERROR.
    pub fn is_ssl_connect_error(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_CONNECT_ERROR
    }

    /// Returns whether this error corresponds to CURLE_BAD_DOWNLOAD_RESUME.
    pub fn is_bad_download_resume(&self) -> bool {
        self.code == curl_sys::CURLE_BAD_DOWNLOAD_RESUME
    }

    /// Returns whether this error corresponds to CURLE_FILE_COULDNT_READ_FILE.
    pub fn is_file_couldnt_read_file(&self) -> bool {
        self.code == curl_sys::CURLE_FILE_COULDNT_READ_FILE
    }

    /// Returns whether this error corresponds to CURLE_FUNCTION_NOT_FOUND.
    pub fn is_function_not_found(&self) -> bool {
        self.code == curl_sys::CURLE_FUNCTION_NOT_FOUND
    }

    /// Returns whether this error corresponds to CURLE_ABORTED_BY_CALLBACK.
    pub fn is_aborted_by_callback(&self) -> bool {
        self.code == curl_sys::CURLE_ABORTED_BY_CALLBACK
    }

    /// Returns whether this error corresponds to CURLE_BAD_FUNCTION_ARGUMENT.
    pub fn is_bad_function_argument(&self) -> bool {
        self.code == curl_sys::CURLE_BAD_FUNCTION_ARGUMENT
    }

    /// Returns whether this error corresponds to CURLE_INTERFACE_FAILED.
    pub fn is_interface_failed(&self) -> bool {
        self.code == curl_sys::CURLE_INTERFACE_FAILED
    }

    /// Returns whether this error corresponds to CURLE_TOO_MANY_REDIRECTS.
    pub fn is_too_many_redirects(&self) -> bool {
        self.code == curl_sys::CURLE_TOO_MANY_REDIRECTS
    }

    /// Returns whether this error corresponds to CURLE_UNKNOWN_OPTION.
    pub fn is_unknown_option(&self) -> bool {
        self.code == curl_sys::CURLE_UNKNOWN_OPTION
    }

    /// Returns whether this error corresponds to CURLE_PEER_FAILED_VERIFICATION.
    pub fn is_peer_failed_verification(&self) -> bool {
        self.code == curl_sys::CURLE_PEER_FAILED_VERIFICATION
    }

    /// Returns whether this error corresponds to CURLE_GOT_NOTHING.
    pub fn is_got_nothing(&self) -> bool {
        self.code == curl_sys::CURLE_GOT_NOTHING
    }

    /// Returns whether this error corresponds to CURLE_SSL_ENGINE_NOTFOUND.
    pub fn is_ssl_engine_notfound(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_ENGINE_NOTFOUND
    }

    /// Returns whether this error corresponds to CURLE_SSL_ENGINE_SETFAILED.
    pub fn is_ssl_engine_setfailed(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_ENGINE_SETFAILED
    }

    /// Returns whether this error corresponds to CURLE_SEND_ERROR.
    pub fn is_send_error(&self) -> bool {
        self.code == curl_sys::CURLE_SEND_ERROR
    }

    /// Returns whether this error corresponds to CURLE_RECV_ERROR.
    pub fn is_recv_error(&self) -> bool {
        self.code == curl_sys::CURLE_RECV_ERROR
    }

    /// Returns whether this error corresponds to CURLE_SSL_CERTPROBLEM.
    pub fn is_ssl_certproblem(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_CERTPROBLEM
    }

    /// Returns whether this error corresponds to CURLE_SSL_CIPHER.
    pub fn is_ssl_cipher(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_CIPHER
    }

    /// Returns whether this error corresponds to CURLE_SSL_CACERT.
    pub fn is_ssl_cacert(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_CACERT
    }

    /// Returns whether this error corresponds to CURLE_BAD_CONTENT_ENCODING.
    pub fn is_bad_content_encoding(&self) -> bool {
        self.code == curl_sys::CURLE_BAD_CONTENT_ENCODING
    }

    /// Returns whether this error corresponds to CURLE_FILESIZE_EXCEEDED.
    pub fn is_filesize_exceeded(&self) -> bool {
        self.code == curl_sys::CURLE_FILESIZE_EXCEEDED
    }

    /// Returns whether this error corresponds to CURLE_USE_SSL_FAILED.
    pub fn is_use_ssl_failed(&self) -> bool {
        self.code == curl_sys::CURLE_USE_SSL_FAILED
    }

    /// Returns whether this error corresponds to CURLE_SEND_FAIL_REWIND.
    pub fn is_send_fail_rewind(&self) -> bool {
        self.code == curl_sys::CURLE_SEND_FAIL_REWIND
    }

    /// Returns whether this error corresponds to CURLE_SSL_ENGINE_INITFAILED.
    pub fn is_ssl_engine_initfailed(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_ENGINE_INITFAILED
    }

    /// Returns whether this error corresponds to CURLE_LOGIN_DENIED.
    pub fn is_login_denied(&self) -> bool {
        self.code == curl_sys::CURLE_LOGIN_DENIED
    }

    /// Returns whether this error corresponds to CURLE_CONV_FAILED.
    pub fn is_conv_failed(&self) -> bool {
        self.code == curl_sys::CURLE_CONV_FAILED
    }

    /// Returns whether this error corresponds to CURLE_CONV_REQD.
    pub fn is_conv_required(&self) -> bool {
        self.code == curl_sys::CURLE_CONV_REQD
    }

    /// Returns whether this error corresponds to CURLE_SSL_CACERT_BADFILE.
    pub fn is_ssl_cacert_badfile(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_CACERT_BADFILE
    }

    /// Returns whether this error corresponds to CURLE_SSL_CRL_BADFILE.
    pub fn is_ssl_crl_badfile(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_CRL_BADFILE
    }

    /// Returns whether this error corresponds to CURLE_SSL_SHUTDOWN_FAILED.
    pub fn is_ssl_shutdown_failed(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_SHUTDOWN_FAILED
    }

    /// Returns whether this error corresponds to CURLE_AGAIN.
    pub fn is_again(&self) -> bool {
        self.code == curl_sys::CURLE_AGAIN
    }

    /// Returns whether this error corresponds to CURLE_SSL_ISSUER_ERROR.
    pub fn is_ssl_issuer_error(&self) -> bool {
        self.code == curl_sys::CURLE_SSL_ISSUER_ERROR
    }

    /// Returns whether this error corresponds to CURLE_CHUNK_FAILED.
    pub fn is_chunk_failed(&self) -> bool {
        self.code == curl_sys::CURLE_CHUNK_FAILED
    }

    /// Returns whether this error corresponds to CURLE_HTTP2.
    pub fn is_http2_error(&self) -> bool {
        self.code == curl_sys::CURLE_HTTP2
    }

    /// Returns whether this error corresponds to CURLE_HTTP2_STREAM.
    pub fn is_http2_stream_error(&self) -> bool {
        self.code == curl_sys::CURLE_HTTP2_STREAM
    }

    // /// Returns whether this error corresponds to CURLE_NO_CONNECTION_AVAILABLE.
    // pub fn is_no_connection_available(&self) -> bool {
    //     self.code == curl_sys::CURLE_NO_CONNECTION_AVAILABLE
    // }

    /// Returns the value of the underlying error corresponding to libcurl.
    pub fn code(&self) -> curl_sys::CURLcode {
        self.code
    }

    /// Returns the general description of this error code, using curl's
    /// builtin `strerror`-like functionality.
    pub fn description(&self) -> &str {
        unsafe {
            let s = curl_sys::curl_easy_strerror(self.code);
            assert!(!s.is_null());
            str::from_utf8(CStr::from_ptr(s).to_bytes()).unwrap()
        }
    }

    /// Returns the extra description of this error, if any is available.
    pub fn extra_description(&self) -> Option<&str> {
        self.extra.as_deref()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = self.description();
        match self.extra {
            Some(ref s) => write!(f, "[{}] {} ({})", self.code(), desc, s),
            None => write!(f, "[{}] {}", self.code(), desc),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Error")
            .field("description", &self.description())
            .field("code", &self.code)
            .field("extra", &self.extra)
            .finish()
    }
}

impl error::Error for Error {}

/// An error returned from "share" operations.
///
/// This structure wraps a `CURLSHcode`.
#[derive(Clone, PartialEq)]
pub struct ShareError {
    code: curl_sys::CURLSHcode,
}

impl ShareError {
    /// Creates a new error from the underlying code returned by libcurl.
    pub fn new(code: curl_sys::CURLSHcode) -> ShareError {
        ShareError { code }
    }

    /// Returns whether this error corresponds to CURLSHE_BAD_OPTION.
    pub fn is_bad_option(&self) -> bool {
        self.code == curl_sys::CURLSHE_BAD_OPTION
    }

    /// Returns whether this error corresponds to CURLSHE_IN_USE.
    pub fn is_in_use(&self) -> bool {
        self.code == curl_sys::CURLSHE_IN_USE
    }

    /// Returns whether this error corresponds to CURLSHE_INVALID.
    pub fn is_invalid(&self) -> bool {
        self.code == curl_sys::CURLSHE_INVALID
    }

    /// Returns whether this error corresponds to CURLSHE_NOMEM.
    pub fn is_nomem(&self) -> bool {
        self.code == curl_sys::CURLSHE_NOMEM
    }

    // /// Returns whether this error corresponds to CURLSHE_NOT_BUILT_IN.
    // pub fn is_not_built_in(&self) -> bool {
    //     self.code == curl_sys::CURLSHE_NOT_BUILT_IN
    // }

    /// Returns the value of the underlying error corresponding to libcurl.
    pub fn code(&self) -> curl_sys::CURLSHcode {
        self.code
    }

    /// Returns curl's human-readable version of this error.
    pub fn description(&self) -> &str {
        unsafe {
            let s = curl_sys::curl_share_strerror(self.code);
            assert!(!s.is_null());
            str::from_utf8(CStr::from_ptr(s).to_bytes()).unwrap()
        }
    }
}

impl fmt::Display for ShareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl fmt::Debug for ShareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ShareError {{ description: {:?}, code: {} }}",
            self.description(),
            self.code
        )
    }
}

impl error::Error for ShareError {}

/// An error from "multi" operations.
///
/// THis structure wraps a `CURLMcode`.
#[derive(Clone, PartialEq)]
pub struct MultiError {
    code: curl_sys::CURLMcode,
}

impl MultiError {
    /// Creates a new error from the underlying code returned by libcurl.
    pub fn new(code: curl_sys::CURLMcode) -> MultiError {
        MultiError { code }
    }

    /// Returns whether this error corresponds to CURLM_BAD_HANDLE.
    pub fn is_bad_handle(&self) -> bool {
        self.code == curl_sys::CURLM_BAD_HANDLE
    }

    /// Returns whether this error corresponds to CURLM_BAD_EASY_HANDLE.
    pub fn is_bad_easy_handle(&self) -> bool {
        self.code == curl_sys::CURLM_BAD_EASY_HANDLE
    }

    /// Returns whether this error corresponds to CURLM_OUT_OF_MEMORY.
    pub fn is_out_of_memory(&self) -> bool {
        self.code == curl_sys::CURLM_OUT_OF_MEMORY
    }

    /// Returns whether this error corresponds to CURLM_INTERNAL_ERROR.
    pub fn is_internal_error(&self) -> bool {
        self.code == curl_sys::CURLM_INTERNAL_ERROR
    }

    /// Returns whether this error corresponds to CURLM_BAD_SOCKET.
    pub fn is_bad_socket(&self) -> bool {
        self.code == curl_sys::CURLM_BAD_SOCKET
    }

    /// Returns whether this error corresponds to CURLM_UNKNOWN_OPTION.
    pub fn is_unknown_option(&self) -> bool {
        self.code == curl_sys::CURLM_UNKNOWN_OPTION
    }

    /// Returns whether this error corresponds to CURLM_CALL_MULTI_PERFORM.
    pub fn is_call_perform(&self) -> bool {
        self.code == curl_sys::CURLM_CALL_MULTI_PERFORM
    }

    // /// Returns whether this error corresponds to CURLM_ADDED_ALREADY.
    // pub fn is_added_already(&self) -> bool {
    //     self.code == curl_sys::CURLM_ADDED_ALREADY
    // }

    /// Returns the value of the underlying error corresponding to libcurl.
    pub fn code(&self) -> curl_sys::CURLMcode {
        self.code
    }

    /// Returns curl's human-readable description of this error.
    pub fn description(&self) -> &str {
        unsafe {
            let s = curl_sys::curl_multi_strerror(self.code);
            assert!(!s.is_null());
            str::from_utf8(CStr::from_ptr(s).to_bytes()).unwrap()
        }
    }
}

impl fmt::Display for MultiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl fmt::Debug for MultiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MultiError")
            .field("description", &self.description())
            .field("code", &self.code)
            .finish()
    }
}

impl error::Error for MultiError {}

/// An error from "form add" operations.
///
/// THis structure wraps a `CURLFORMcode`.
#[derive(Clone, PartialEq)]
pub struct FormError {
    code: curl_sys::CURLFORMcode,
}

impl FormError {
    /// Creates a new error from the underlying code returned by libcurl.
    pub fn new(code: curl_sys::CURLFORMcode) -> FormError {
        FormError { code }
    }

    /// Returns whether this error corresponds to CURL_FORMADD_MEMORY.
    pub fn is_memory(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_MEMORY
    }

    /// Returns whether this error corresponds to CURL_FORMADD_OPTION_TWICE.
    pub fn is_option_twice(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_OPTION_TWICE
    }

    /// Returns whether this error corresponds to CURL_FORMADD_NULL.
    pub fn is_null(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_NULL
    }

    /// Returns whether this error corresponds to CURL_FORMADD_UNKNOWN_OPTION.
    pub fn is_unknown_option(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_UNKNOWN_OPTION
    }

    /// Returns whether this error corresponds to CURL_FORMADD_INCOMPLETE.
    pub fn is_incomplete(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_INCOMPLETE
    }

    /// Returns whether this error corresponds to CURL_FORMADD_ILLEGAL_ARRAY.
    pub fn is_illegal_array(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_ILLEGAL_ARRAY
    }

    /// Returns whether this error corresponds to CURL_FORMADD_DISABLED.
    pub fn is_disabled(&self) -> bool {
        self.code == curl_sys::CURL_FORMADD_DISABLED
    }

    /// Returns the value of the underlying error corresponding to libcurl.
    pub fn code(&self) -> curl_sys::CURLFORMcode {
        self.code
    }

    /// Returns a human-readable description of this error code.
    pub fn description(&self) -> &str {
        match self.code {
            curl_sys::CURL_FORMADD_MEMORY => "allocation failure",
            curl_sys::CURL_FORMADD_OPTION_TWICE => "one option passed twice",
            curl_sys::CURL_FORMADD_NULL => "null pointer given for string",
            curl_sys::CURL_FORMADD_UNKNOWN_OPTION => "unknown option",
            curl_sys::CURL_FORMADD_INCOMPLETE => "form information not complete",
            curl_sys::CURL_FORMADD_ILLEGAL_ARRAY => "illegal array in option",
            curl_sys::CURL_FORMADD_DISABLED => {
                "libcurl does not have support for this option compiled in"
            }
            _ => "unknown form error",
        }
    }
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl fmt::Debug for FormError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FormError")
            .field("description", &self.description())
            .field("code", &self.code)
            .finish()
    }
}

impl error::Error for FormError {}

impl From<ffi::NulError> for Error {
    fn from(_: ffi::NulError) -> Error {
        Error {
            code: curl_sys::CURLE_CONV_FAILED,
            extra: None,
        }
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl From<ShareError> for io::Error {
    fn from(e: ShareError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl From<MultiError> for io::Error {
    fn from(e: MultiError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl From<FormError> for io::Error {
    fn from(e: FormError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}
