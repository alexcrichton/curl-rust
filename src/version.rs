use std::ffi::CStr;
use std::fmt;
use std::str;

use curl_sys;
use libc::{c_char, c_int};

/// Version information about libcurl and the capabilities that it supports.
pub struct Version {
    inner: *mut curl_sys::curl_version_info_data,
}

unsafe impl Send for Version {}
unsafe impl Sync for Version {}

/// An iterator over the list of protocols a version supports.
#[derive(Clone)]
pub struct Protocols<'a> {
    cur: *const *const c_char,
    _inner: &'a Version,
}

impl Version {
    /// Returns the libcurl version that this library is currently linked against.
    pub fn num() -> &'static str {
        unsafe {
            let s = CStr::from_ptr(curl_sys::curl_version() as *const _);
            str::from_utf8(s.to_bytes()).unwrap()
        }
    }

    /// Returns the libcurl version that this library is currently linked against.
    pub fn get() -> Version {
        unsafe {
            let ptr = curl_sys::curl_version_info(curl_sys::CURLVERSION_NOW);
            assert!(!ptr.is_null());
            Version { inner: ptr }
        }
    }

    /// Returns the human readable version string,
    pub fn version(&self) -> &str {
        unsafe { ::opt_str((*self.inner).version).unwrap() }
    }

    /// Returns a numeric representation of the version number
    ///
    /// This is a 24 bit number made up of the major number, minor, and then
    /// patch number. For example 7.9.8 will return 0x070908.
    pub fn version_num(&self) -> u32 {
        unsafe { (*self.inner).version_num as u32 }
    }

    /// Returns a human readable string of the host libcurl is built for.
    ///
    /// This is discovered as part of the build environment.
    pub fn host(&self) -> &str {
        unsafe { ::opt_str((*self.inner).host).unwrap() }
    }

    /// Returns whether libcurl supports IPv6
    pub fn feature_ipv6(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_IPV6)
    }

    /// Returns whether libcurl supports SSL
    pub fn feature_ssl(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_SSL)
    }

    /// Returns whether libcurl supports HTTP deflate via libz
    pub fn feature_libz(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_LIBZ)
    }

    /// Returns whether libcurl supports HTTP NTLM
    pub fn feature_ntlm(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_NTLM)
    }

    /// Returns whether libcurl supports HTTP GSSNEGOTIATE
    pub fn feature_gss_negotiate(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_GSSNEGOTIATE)
    }

    /// Returns whether libcurl was built with debug capabilities
    pub fn feature_debug(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_DEBUG)
    }

    /// Returns whether libcurl was built with SPNEGO authentication
    pub fn feature_spnego(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_SPNEGO)
    }

    /// Returns whether libcurl was built with large file support
    pub fn feature_largefile(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_LARGEFILE)
    }

    /// Returns whether libcurl was built with support for IDNA, domain names
    /// with international letters.
    pub fn feature_idn(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_IDN)
    }

    /// Returns whether libcurl was built with support for SSPI.
    pub fn feature_sspi(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_SSPI)
    }

    /// Returns whether libcurl was built with asynchronous name lookups.
    pub fn feature_async_dns(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_ASYNCHDNS)
    }

    /// Returns whether libcurl was built with support for character
    /// conversions.
    pub fn feature_conv(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_CONV)
    }

    /// Returns whether libcurl was built with support for TLS-SRP.
    pub fn feature_tlsauth_srp(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_TLSAUTH_SRP)
    }

    /// Returns whether libcurl was built with support for NTLM delegation to
    /// winbind helper.
    pub fn feature_ntlm_wb(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_NTLM_WB)
    }

    /// Returns whether libcurl was built with support for unix domain socket
    pub fn feature_unix_domain_socket(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_UNIX_SOCKETS)
    }

    /// Returns whether libcurl was built with support for HTTP2.
    pub fn feature_http2(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_HTTP2)
    }

    /// Returns whether libcurl was built with support for HTTP3.
    pub fn feature_http3(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_HTTP3)
    }

    /// Returns whether libcurl was built with support for Brotli.
    pub fn feature_brotli(&self) -> bool {
        self.flag(curl_sys::CURL_VERSION_BROTLI)
    }

    fn flag(&self, flag: c_int) -> bool {
        unsafe { (*self.inner).features & flag != 0 }
    }

    /// Returns the version of OpenSSL that is used, or None if there is no SSL
    /// support.
    pub fn ssl_version(&self) -> Option<&str> {
        unsafe { ::opt_str((*self.inner).ssl_version) }
    }

    /// Returns the version of libz that is used, or None if there is no libz
    /// support.
    pub fn libz_version(&self) -> Option<&str> {
        unsafe { ::opt_str((*self.inner).libz_version) }
    }

    /// Returns an iterator over the list of protocols that this build of
    /// libcurl supports.
    pub fn protocols(&self) -> Protocols {
        unsafe {
            Protocols {
                _inner: self,
                cur: (*self.inner).protocols,
            }
        }
    }

    /// If available, the human readable version of ares that libcurl is linked
    /// against.
    pub fn ares_version(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SECOND {
                ::opt_str((*self.inner).ares)
            } else {
                None
            }
        }
    }

    /// If available, the version of ares that libcurl is linked against.
    pub fn ares_version_num(&self) -> Option<u32> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SECOND {
                Some((*self.inner).ares_num as u32)
            } else {
                None
            }
        }
    }

    /// If available, the version of libidn that libcurl is linked against.
    pub fn libidn_version(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_THIRD {
                ::opt_str((*self.inner).libidn)
            } else {
                None
            }
        }
    }

    /// If available, the version of iconv libcurl is linked against.
    pub fn iconv_version_num(&self) -> Option<u32> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_FOURTH {
                Some((*self.inner).iconv_ver_num as u32)
            } else {
                None
            }
        }
    }

    /// If available, the version of libssh that libcurl is linked against.
    pub fn libssh_version(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_FOURTH {
                ::opt_str((*self.inner).libssh_version)
            } else {
                None
            }
        }
    }

    /// If available, the version of brotli libcurl is linked against.
    pub fn brotli_version_num(&self) -> Option<u32> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_FIFTH {
                Some((*self.inner).brotli_ver_num)
            } else {
                None
            }
        }
    }

    /// If available, the version of brotli libcurl is linked against.
    pub fn brotli_version(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_FIFTH {
                ::opt_str((*self.inner).brotli_version)
            } else {
                None
            }
        }
    }

    /// If available, the version of nghttp2 libcurl is linked against.
    pub fn nghttp2_version_num(&self) -> Option<u32> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SIXTH {
                Some((*self.inner).nghttp2_ver_num)
            } else {
                None
            }
        }
    }

    /// If available, the version of nghttp2 libcurl is linked against.
    pub fn nghttp2_version(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SIXTH {
                ::opt_str((*self.inner).nghttp2_version)
            } else {
                None
            }
        }
    }

    /// If available, the version of quic libcurl is linked against.
    pub fn quic_version(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SIXTH {
                ::opt_str((*self.inner).quic_version)
            } else {
                None
            }
        }
    }

    /// If available, the built-in default of CURLOPT_CAINFO.
    pub fn cainfo(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SEVENTH {
                ::opt_str((*self.inner).cainfo)
            } else {
                None
            }
        }
    }

    /// If available, the built-in default of CURLOPT_CAPATH.
    pub fn capath(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).age >= curl_sys::CURLVERSION_SEVENTH {
                ::opt_str((*self.inner).capath)
            } else {
                None
            }
        }
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Version");
        f.field("version", &self.version())
            .field("host", &self.host())
            .field("feature_ipv6", &self.feature_ipv6())
            .field("feature_ssl", &self.feature_ssl())
            .field("feature_libz", &self.feature_libz())
            .field("feature_ntlm", &self.feature_ntlm())
            .field("feature_gss_negotiate", &self.feature_gss_negotiate())
            .field("feature_debug", &self.feature_debug())
            .field("feature_spnego", &self.feature_spnego())
            .field("feature_largefile", &self.feature_largefile())
            .field("feature_idn", &self.feature_idn())
            .field("feature_sspi", &self.feature_sspi())
            .field("feature_async_dns", &self.feature_async_dns())
            .field("feature_conv", &self.feature_conv())
            .field("feature_tlsauth_srp", &self.feature_tlsauth_srp())
            .field("feature_ntlm_wb", &self.feature_ntlm_wb())
            .field(
                "feature_unix_domain_socket",
                &self.feature_unix_domain_socket(),
            );

        if let Some(s) = self.ssl_version() {
            f.field("ssl_version", &s);
        }
        if let Some(s) = self.libz_version() {
            f.field("libz_version", &s);
        }
        if let Some(s) = self.ares_version() {
            f.field("ares_version", &s);
        }
        if let Some(s) = self.libidn_version() {
            f.field("libidn_version", &s);
        }
        if let Some(s) = self.iconv_version_num() {
            f.field("iconv_version_num", &format!("{:x}", s));
        }
        if let Some(s) = self.libssh_version() {
            f.field("libssh_version", &s);
        }
        if let Some(s) = self.brotli_version_num() {
            f.field("brotli_version_num", &format!("{:x}", s));
        }
        if let Some(s) = self.brotli_version() {
            f.field("brotli_version", &s);
        }
        if let Some(s) = self.nghttp2_version_num() {
            f.field("nghttp2_version_num", &format!("{:x}", s));
        }
        if let Some(s) = self.nghttp2_version() {
            f.field("nghttp2_version", &s);
        }
        if let Some(s) = self.quic_version() {
            f.field("quic_version", &s);
        }

        f.field("protocols", &self.protocols().collect::<Vec<_>>());

        f.finish()
    }
}

impl<'a> Iterator for Protocols<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        unsafe {
            if (*self.cur).is_null() {
                return None;
            }
            let ret = ::opt_str(*self.cur).unwrap();
            self.cur = self.cur.offset(1);
            Some(ret)
        }
    }
}

impl<'a> fmt::Debug for Protocols<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
