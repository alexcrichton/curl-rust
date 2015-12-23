use std::ffi::CStr;
use std::error;
use std::fmt;
use std::str;

use curl_ffi as ffi;

pub use curl_ffi::CURLcode::CURLE_OK as OK;
pub use curl_ffi::CURLcode::CURLE_UNSUPPORTED_PROTOCOL as UNSUPPORTED_PROTOCOL;
pub use curl_ffi::CURLcode::CURLE_FAILED_INIT as FAILED_INIT;
pub use curl_ffi::CURLcode::CURLE_URL_MALFORMAT as URL_MALFORMAT;
pub use curl_ffi::CURLcode::CURLE_NOT_BUILT_IN as NOT_BUILT_IN;
pub use curl_ffi::CURLcode::CURLE_COULDNT_RESOLVE_PROXY as COULDNT_RESOLVE_PROXY;
pub use curl_ffi::CURLcode::CURLE_COULDNT_RESOLVE_HOST as COULDNT_RESOLVE_HOST;
pub use curl_ffi::CURLcode::CURLE_COULDNT_CONNECT as COULDNT_CONNECT;
pub use curl_ffi::CURLcode::CURLE_FTP_WEIRD_SERVER_REPLY as FTP_WEIRD_SERVER_REPLY;
pub use curl_ffi::CURLcode::CURLE_REMOTE_ACCESS_DENIED as REMOTE_ACCESS_DENIED;
pub use curl_ffi::CURLcode::CURLE_FTP_ACCEPT_FAILED as FTP_ACCEPT_FAILED;
pub use curl_ffi::CURLcode::CURLE_FTP_WEIRD_PASS_REPLY as FTP_WEIRD_PASS_REPLY;
pub use curl_ffi::CURLcode::CURLE_FTP_ACCEPT_TIMEOUT as FTP_ACCEPT_TIMEOUT;
pub use curl_ffi::CURLcode::CURLE_FTP_WEIRD_PASV_REPLY as FTP_WEIRD_PASV_REPLY;
pub use curl_ffi::CURLcode::CURLE_FTP_WEIRD_227_FORMAT as FTP_WEIRD_227_FORMAT;
pub use curl_ffi::CURLcode::CURLE_FTP_CANT_GET_HOST as FTP_CANT_GET_HOST;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE16 as OBSOLETE16;
pub use curl_ffi::CURLcode::CURLE_FTP_COULDNT_SET_TYPE as FTP_COULDNT_SET_TYPE;
pub use curl_ffi::CURLcode::CURLE_PARTIAL_FILE as PARTIAL_FILE;
pub use curl_ffi::CURLcode::CURLE_FTP_COULDNT_RETR_FILE as FTP_COULDNT_RETR_FILE;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE20 as OBSOLETE20;
pub use curl_ffi::CURLcode::CURLE_QUOTE_ERROR as QUOTE_ERROR;
pub use curl_ffi::CURLcode::CURLE_HTTP_RETURNED_ERROR as HTTP_RETURNED_ERROR;
pub use curl_ffi::CURLcode::CURLE_WRITE_ERROR as WRITE_ERROR;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE24 as OBSOLETE24;
pub use curl_ffi::CURLcode::CURLE_UPLOAD_FAILED as UPLOAD_FAILED;
pub use curl_ffi::CURLcode::CURLE_READ_ERROR as READ_ERROR;
pub use curl_ffi::CURLcode::CURLE_OUT_OF_MEMORY as OUT_OF_MEMORY;
pub use curl_ffi::CURLcode::CURLE_OPERATION_TIMEDOUT as OPERATION_TIMEDOUT;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE29 as OBSOLETE29;
pub use curl_ffi::CURLcode::CURLE_FTP_PORT_FAILED as FTP_PORT_FAILED;
pub use curl_ffi::CURLcode::CURLE_FTP_COULDNT_USE_REST as FTP_COULDNT_USE_REST;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE32 as OBSOLETE32;
pub use curl_ffi::CURLcode::CURLE_RANGE_ERROR as RANGE_ERROR;
pub use curl_ffi::CURLcode::CURLE_HTTP_POST_ERROR as HTTP_POST_ERROR;
pub use curl_ffi::CURLcode::CURLE_SSL_CONNECT_ERROR as SSL_CONNECT_ERROR;
pub use curl_ffi::CURLcode::CURLE_BAD_DOWNLOAD_RESUME as BAD_DOWNLOAD_RESUME;
pub use curl_ffi::CURLcode::CURLE_FILE_COULDNT_READ_FILE as FILE_COULDNT_READ_FILE;
pub use curl_ffi::CURLcode::CURLE_LDAP_CANNOT_BIND as LDAP_CANNOT_BIND;
pub use curl_ffi::CURLcode::CURLE_LDAP_SEARCH_FAILED as LDAP_SEARCH_FAILED;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE40 as OBSOLETE40;
pub use curl_ffi::CURLcode::CURLE_FUNCTION_NOT_FOUND as FUNCTION_NOT_FOUND;
pub use curl_ffi::CURLcode::CURLE_ABORTED_BY_CALLBACK as ABORTED_BY_CALLBACK;
pub use curl_ffi::CURLcode::CURLE_BAD_FUNCTION_ARGUMENT as BAD_FUNCTION_ARGUMENT;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE44 as OBSOLETE44;
pub use curl_ffi::CURLcode::CURLE_INTERFACE_FAILED as INTERFACE_FAILED;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE46 as OBSOLETE46;
pub use curl_ffi::CURLcode::CURLE_TOO_MANY_REDIRECTS  as TOO_MANY_REDIRECTS ;
pub use curl_ffi::CURLcode::CURLE_UNKNOWN_OPTION as UNKNOWN_OPTION;
pub use curl_ffi::CURLcode::CURLE_TELNET_OPTION_SYNTAX  as TELNET_OPTION_SYNTAX ;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE50 as OBSOLETE50;
pub use curl_ffi::CURLcode::CURLE_PEER_FAILED_VERIFICATION as PEER_FAILED_VERIFICATION;
pub use curl_ffi::CURLcode::CURLE_GOT_NOTHING as GOT_NOTHING;
pub use curl_ffi::CURLcode::CURLE_SSL_ENGINE_NOTFOUND as SSL_ENGINE_NOTFOUND;
pub use curl_ffi::CURLcode::CURLE_SSL_ENGINE_SETFAILED as SSL_ENGINE_SETFAILED;
pub use curl_ffi::CURLcode::CURLE_SEND_ERROR as SEND_ERROR;
pub use curl_ffi::CURLcode::CURLE_RECV_ERROR as RECV_ERROR;
pub use curl_ffi::CURLcode::CURLE_OBSOLETE57 as OBSOLETE57;
pub use curl_ffi::CURLcode::CURLE_SSL_CERTPROBLEM as SSL_CERTPROBLEM;
pub use curl_ffi::CURLcode::CURLE_SSL_CIPHER as SSL_CIPHER;
pub use curl_ffi::CURLcode::CURLE_SSL_CACERT as SSL_CACERT;
pub use curl_ffi::CURLcode::CURLE_BAD_CONTENT_ENCODING as BAD_CONTENT_ENCODING;
pub use curl_ffi::CURLcode::CURLE_LDAP_INVALID_URL as LDAP_INVALID_URL;
pub use curl_ffi::CURLcode::CURLE_FILESIZE_EXCEEDED as FILESIZE_EXCEEDED;
pub use curl_ffi::CURLcode::CURLE_USE_SSL_FAILED as USE_SSL_FAILED;
pub use curl_ffi::CURLcode::CURLE_SEND_FAIL_REWIND as SEND_FAIL_REWIND;
pub use curl_ffi::CURLcode::CURLE_SSL_ENGINE_INITFAILED as SSL_ENGINE_INITFAILED;
pub use curl_ffi::CURLcode::CURLE_LOGIN_DENIED as LOGIN_DENIED;
pub use curl_ffi::CURLcode::CURLE_TFTP_NOTFOUND as TFTP_NOTFOUND;
pub use curl_ffi::CURLcode::CURLE_TFTP_PERM as TFTP_PERM;
pub use curl_ffi::CURLcode::CURLE_REMOTE_DISK_FULL as REMOTE_DISK_FULL;
pub use curl_ffi::CURLcode::CURLE_TFTP_ILLEGAL as TFTP_ILLEGAL;
pub use curl_ffi::CURLcode::CURLE_TFTP_UNKNOWNID as TFTP_UNKNOWNID;
pub use curl_ffi::CURLcode::CURLE_REMOTE_FILE_EXISTS as REMOTE_FILE_EXISTS;
pub use curl_ffi::CURLcode::CURLE_TFTP_NOSUCHUSER as TFTP_NOSUCHUSER;
pub use curl_ffi::CURLcode::CURLE_CONV_FAILED as CONV_FAILED;
pub use curl_ffi::CURLcode::CURLE_CONV_REQD as CONV_REQD;
pub use curl_ffi::CURLcode::CURLE_SSL_CACERT_BADFILE as SSL_CACERT_BADFILE;
pub use curl_ffi::CURLcode::CURLE_REMOTE_FILE_NOT_FOUND as REMOTE_FILE_NOT_FOUND;
pub use curl_ffi::CURLcode::CURLE_SSH as SSH;
pub use curl_ffi::CURLcode::CURLE_SSL_SHUTDOWN_FAILED as SSL_SHUTDOWN_FAILED;
pub use curl_ffi::CURLcode::CURLE_AGAIN as AGAIN;
pub use curl_ffi::CURLcode::CURLE_SSL_CRL_BADFILE as SSL_CRL_BADFILE;
pub use curl_ffi::CURLcode::CURLE_SSL_ISSUER_ERROR as SSL_ISSUER_ERROR;
pub use curl_ffi::CURLcode::CURLE_FTP_PRET_FAILED as FTP_PRET_FAILED;
pub use curl_ffi::CURLcode::CURLE_RTSP_CSEQ_ERROR as RTSP_CSEQ_ERROR;
pub use curl_ffi::CURLcode::CURLE_RTSP_SESSION_ERROR as RTSP_SESSION_ERROR;
pub use curl_ffi::CURLcode::CURLE_FTP_BAD_FILE_LIST as FTP_BAD_FILE_LIST;
pub use curl_ffi::CURLcode::CURLE_CHUNK_FAILED as CHUNK_FAILED;
pub use curl_ffi::CURLcode::CURLE_NO_CONNECTION_AVAILABLE as NO_CONNECTION_AVAILABLE;
pub use curl_ffi::CURLcode::CURLE_LAST as LAST;

#[derive(Copy, Clone)]
pub struct ErrCode(pub ffi::CURLcode);

impl ErrCode {
    pub fn is_success(self) -> bool {
       self.code() as i32 == OK as i32
    }

    pub fn code(self) -> ffi::CURLcode { let ErrCode(c) = self; c }
}

impl fmt::Debug for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ErrCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe {
            let ptr = ffi::curl_easy_strerror(self.code());
            CStr::from_ptr(ptr as *const _).to_bytes()
        };

        match str::from_utf8(s) {
            Ok(s) => write!(fmt, "{}", s),
            Err(err) => write!(fmt, "{}", err)
        }
    }
}

impl error::Error for ErrCode {
    fn description(&self) -> &str {
        let code = self.code();
        let s = unsafe {
            CStr::from_ptr(ffi::curl_easy_strerror(code) as *const _).to_bytes()
        };
        str::from_utf8(s).unwrap()
    }
}
