#![allow(bad_style)]
#![doc(html_root_url = "https://docs.rs/curl-sys/0.3")]

extern crate curl_sys;
extern crate libc;

pub use curl_sys::*;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
use winapi::winsock2::fd_set;
#[cfg(windows)]
use winapi::{SOCKADDR, c_int, c_uint};

#[repr(C)]
#[cfg(windows)]
pub struct curl_sockaddr {
    pub family: c_int,
    pub socktype: c_int,
    pub protocol: c_int,
    pub addrlen: c_uint,
    pub addr: SOCKADDR,
}

pub type curl_opensocket_callback = extern fn(*mut libc::c_void,
                                              curlsocktype,
                                              *mut curl_sockaddr) -> curl_socket_t;

#[cfg(windows)]
extern {
    pub fn curl_multi_fdset(multi_handle: *mut CURLM,
                            read_fd_set: *mut fd_set,
                            write_fd_set: *mut fd_set,
                            exc_fd_set: *mut fd_set,
                            max_fd: *mut c_int) -> CURLMcode;
}
