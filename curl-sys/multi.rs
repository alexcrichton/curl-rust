#![allow(non_camel_case_types, raw_pointer_derive)]

extern crate libc;
#[cfg(not(target_env = "msvc"))]
extern crate libz_sys;
#[cfg(unix)]
extern crate openssl_sys;

use libc::{c_void, c_int, c_char, c_uint, c_long};

pub type CURL = c_void;
pub type CURLM = c_void;
pub type CURLMoption = c_int;
pub type C = c_int;


pub const CURLOPTTYPE_LONG: c_int = 0;
pub const CURLOPTTYPE_OBJECTPOINT: c_int = 10_000;
pub const CURLOPTTYPE_FUNCTIONPOINT: c_int = 20_000;
pub const CURLOPTTYPE_OFF_T: c_int = 30_000;


macro_rules! DEFOPTM {
    ($name:ident, $ty:ident, $num:expr) => (
        #[allow(dead_code)]
        pub const $name: CURLMoption = $ty + $num;
    )
}

macro_rules! ALIASM {
    ($name:ident, $to:ident) => (
        #[allow(dead_code)]
        pub const $name: CURLMoption = $to;
    )
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CURLMcode {
    CURLM_CALL_MULTI_PERFORM = -1, /* please call curl_multi_perform() or
                                    curl_multi_socket*() soon */
    CURLM_OK,
    CURLM_BAD_HANDLE, /* the passed-in handle is not a valid CURLM handle */
    CURLM_BAD_EASY_HANDLE, /* an easy handle was not good/valid */
    CURLM_OUT_OF_MEMORY, /* if you ever get this, you're in deep sh*t */
    CURLM_INTERNAL_ERROR, /* this is a libcurl bug */
    CURLM_BAD_SOCKET, /* the passed in socket argument did not match */
    CURLM_UNKNOWN_OPTION, /* curl_multi_setopt() with unsupported option */
    CURLM_ADDED_ALREADY, /* an easy handle already added to a multi handle was
                            attempted to get added - again */
    CURLM_LAST
}

/* This is the socket callback function pointer */
DEFOPTM!(CURLMOPT_SOCKETFUNCTION, FUNCTIONPOINT, 1);
/* This is the argument passed to the socket callback */
DEFOPTM!(CURLMOPT_SOCKETDATA, OBJECTPOINT, 2);

/* set to 1 to enable pipelining for this multi handle */
DEFOPTM!(PIPELINING, LONG, 3);

/* This is the timer callback function pointer */
DEFOPTM!(TIMERFUNCTION, FUNCTIONPOINT, 4);

/* This is the argument passed to the timer callback */
DEFOPTM!(TIMERDATA, OBJECTPOINT, 5);

/* maximum number of entries in the connection cache */
DEFOPTM!(MAXCONNECTS, LONG, 6);

/* maximum number of (pipelining) connections to one host */
DEFOPTM!(MAX_HOST_CONNECTIONS, LONG, 7);

/* maximum number of requests in a pipeline */
DEFOPTM!(MAX_PIPELINE_LENGTH, LONG, 8);

/* a connection with a content-length longer than this
      will not be considered for pipelining */
DEFOPTM!(CONTENT_LENGTH_PENALTY_SIZE, OFF_T, 9);

/* a connection with a chunk length longer than this
      will not be considered for pipelining */
DEFOPTM!(CHUNK_LENGTH_PENALTY_SIZE, OFF_T, 10);

/* a list of site names(+port) that are blacklisted from
      pipelining */
DEFOPTM!(PIPELINING_SITE_BL, OBJECTPOINT, 11);

/* a list of server types that are blacklisted from
      pipelining */
DEFOPTM!(PIPELINING_SERVER_BL, OBJECTPOINT, 12);

/* maximum number of open connections in total */
DEFOPTM!(MAX_TOTAL_CONNECTIONS, LONG, 13);

//CURLMOPT_LASTENTRY /* the last unused */

ALIASM!(LONG, CURLOPTTYPE_LONG);
ALIASM!(OBJECTPOINT, CURLOPTTYPE_OBJECTPOINT);
ALIASM!(FUNCTIONPOINT, CURLOPTTYPE_FUNCTIONPOINT);
ALIASM!(OFF_T, CURLOPTTYPE_OFF_T);

extern {
    // Multi curl mode
    pub fn curl_multi_cleanup(curl: *mut CURLM);
    pub fn curl_multi_add_handle(curl: *mut CURLM, easy: *mut CURL) ->CURLMcode;
    pub fn curl_multi_init() -> *mut CURLM;
    pub fn curl_multi_setopt(curl: *mut CURLM, option: CURLMoption, ...) -> CURLMcode;
    pub fn curl_multi_strerror(code: CURLMcode) -> *const c_char;
}
