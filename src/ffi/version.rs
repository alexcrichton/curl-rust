#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::marker;
use std::ffi::CStr;
use std::{fmt, ptr, str};
use libc::{c_char, c_int};

use curl_ffi as ffi;

#[allow(missing_copy_implementations)]
pub struct Version { inner: *mut ffi::curl_version_info_data }

impl Version {

    pub fn version_str<'a>(&'a self) -> &'a str {
        as_str(unsafe { (*self.inner).version }).unwrap()
    }

    pub fn version_major(&self) -> u32 {
        (unsafe { (*self.inner).version_num } as u32 & 0xFF0000) >> 16
    }

    pub fn version_minor(&self) -> u32 {
        (unsafe { (*self.inner).version_num } as u32 & 0xFF00) >> 8
    }

    pub fn version_patch(&self) -> u32 {
        (unsafe { (*self.inner).version_num } as u32 & 0xFF)
    }

    pub fn host<'a>(&'a self) -> &'a str {
        as_str(unsafe { (*self.inner).host }).unwrap()
    }

    fn features(&self) -> c_int { unsafe { (*self.inner).features } }

    pub fn is_ipv6_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_IPV6) == ffi::CURL_VERSION_IPV6
    }

    pub fn is_kerbos4_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_KERBEROS4) == ffi::CURL_VERSION_KERBEROS4
    }

    pub fn is_ssl_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_SSL) == ffi::CURL_VERSION_SSL
    }

    pub fn is_libz_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_LIBZ) == ffi::CURL_VERSION_LIBZ
    }

    pub fn is_ntlm_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_NTLM) == ffi::CURL_VERSION_NTLM
    }

    pub fn is_gss_negotiate_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_GSSNEGOTIATE) == ffi::CURL_VERSION_GSSNEGOTIATE
    }

    pub fn is_debug_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_DEBUG) == ffi::CURL_VERSION_DEBUG
    }

    pub fn is_async_dns_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_ASYNCHDNS) == ffi::CURL_VERSION_ASYNCHDNS
    }

    pub fn is_spengo_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_SPNEGO) == ffi::CURL_VERSION_SPNEGO
    }

    pub fn is_large_file_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_LARGEFILE) == ffi::CURL_VERSION_LARGEFILE
    }

    pub fn is_idn_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_IDN) == ffi::CURL_VERSION_IDN
    }

    pub fn is_sspi_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_SSPI) == ffi::CURL_VERSION_SSPI
    }

    pub fn is_conv_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_CONV) == ffi::CURL_VERSION_CONV
    }

    pub fn is_curl_debug_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_CURLDEBUG) == ffi::CURL_VERSION_CURLDEBUG
    }

    pub fn is_tls_auth_srp_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_TLSAUTH_SRP) == ffi::CURL_VERSION_TLSAUTH_SRP
    }

    pub fn is_ntlm_wb_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_NTLM_WB) == ffi::CURL_VERSION_NTLM_WB
    }

    pub fn is_http2_enabled(&self) -> bool {
        (self.features() & ffi::CURL_VERSION_HTTP2) == ffi::CURL_VERSION_HTTP2
    }

    pub fn ssl_version<'a>(&'a self) -> Option<&'a str> {
        as_str(unsafe { (*self.inner).ssl_version })
    }

    pub fn libz_version<'a>(&'a self) -> Option<&'a str> {
        as_str(unsafe { (*self.inner).libz_version })
    }

    pub fn protocols<'a>(&'a self) -> Protocols<'a> {
        Protocols {
            curr: unsafe { (*self.inner).protocols },
            _marker: marker::PhantomData
        }
    }

    pub fn ares_version<'a>(&'a self) -> Option<&'a str> {
        as_str(unsafe { (*self.inner).ares })
    }

    pub fn ares_version_num(&self) -> Option<u32> {
        match self.ares_version() {
            Some(_) => Some(unsafe { (*self.inner).ares_num } as u32),
            None => None
        }
    }

    pub fn idn_version<'a>(&'a self) -> Option<&'a str> {
        if self.is_idn_enabled() {
            as_str(unsafe { (*self.inner).libidn })
        }
        else {
            None
        }
    }

    pub fn iconv_version(self) -> Option<u32> {
        if self.is_conv_enabled() {
            Some(unsafe { (*self.inner).iconv_ver_num } as u32)
        }
        else {
            None
        }
    }

    pub fn ssh_version<'a>(&'a self) -> Option<&'a str> {
        as_str(unsafe { (*self.inner).libssh_version })
    }
}

#[derive(Copy, Clone)]
pub struct Protocols<'a> {
    curr: *const *const c_char,
    _marker: marker::PhantomData<&'a str>,
}

impl<'a> Iterator for Protocols<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        unsafe {
            let proto = *self.curr;

            if proto == ptr::null() {
                return None;
            }

            self.curr = self.curr.offset(1);
            as_str(proto)
        }
    }
}

impl<'a> fmt::Display for Protocols<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut i = self.clone();

        try!(write!(fmt, "["));

        match i.next() {
            Some(proto) => try!(write!(fmt, "{}", proto)),
            None => return write!(fmt, "]")
        }

        for proto in i {
            try!(write!(fmt, ", {}", proto));
        }

        write!(fmt, "]")
    }
}

fn as_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p == ptr::null() {
        return None;
    }

    unsafe {
        str::from_utf8(CStr::from_ptr(p as *const _).to_bytes()).ok()
    }
}

pub fn version_info() -> Version {
    Version {
        inner: unsafe { ffi::curl_version_info(ffi::CURL_VERSION_NOW) },
    }
}

pub fn version() -> &'static str {
    unsafe {
        let version = ffi::curl_version();
        str::from_utf8(CStr::from_ptr(version as *const _).to_bytes()).unwrap()
    }
}
