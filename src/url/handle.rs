use std::ffi::{c_char, c_void};
use std::fmt;
use std::ptr::null;

use curl_sys::CURLU;

use super::{Error, GetFlags, SetFlags};

/// A libcurl URL object that holds or can hold URL components for a single URL
pub struct Url {
    raw: *mut CURLU,
}

impl Url {
    /// Allocate a new URL object.
    pub fn new() -> Result<Self, Error> {
        let raw = unsafe { curl_sys::curl_url() };
        assert!(!raw.is_null());
        Ok(Self { raw })
    }

    /// Return the raw pointer to the underlying CURLU handle.
    pub fn as_raw(&self) -> *mut CURLU {
        self.raw
    }

    fn ffi_set(
        &mut self,
        part: curl_sys::CURLUPart,
        value: Option<&str>,
        flags: SetFlags,
    ) -> Result<(), Error> {
        let c_value = value
            .map(|value| {
                std::ffi::CString::new(value)
                    .map_err(|_| Error::new(curl_sys::CURLUE_MALFORMED_INPUT))
            })
            .transpose()?;
        let ptr = c_value.as_ref().map(|s| s.as_ptr()).unwrap_or(null());
        let code = unsafe { curl_sys::curl_url_set(self.raw, part, ptr, flags.0) };
        cvt(code)
    }

    fn ffi_get(
        &self,
        part: curl_sys::CURLUPart,
        flags: GetFlags,
        allowing: curl_sys::CURLUcode,
    ) -> Result<Option<String>, Error> {
        let mut curl_str: *mut c_char = std::ptr::null_mut();
        let code = unsafe { curl_sys::curl_url_get(self.raw, part, &mut curl_str, flags.0) };
        if code != allowing {
            cvt(code)?;
        }
        if curl_str.is_null() {
            return Ok(None);
        }
        struct CurlStr(*mut c_char);
        impl Drop for CurlStr {
            fn drop(&mut self) {
                unsafe { curl_sys::curl_free(self.0 as *mut c_void) };
            }
        }
        let curl_str = CurlStr(curl_str);
        let res = unsafe { std::ffi::CStr::from_ptr(curl_str.0) }.to_str();
        match res {
            Ok(s) => Ok(Some(s.to_owned())),
            Err(_) => Err(Error::new(curl_sys::CURLUE_MALFORMED_INPUT)),
        }
    }

    /// Set the URL to empty.
    pub fn clear_url(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_URL, None, SetFlags::new())
    }

    /// Replace the full URL. If the URL object is already populated, the new
    /// URL can be relative to the previous.
    ///
    /// When successfully setting a new URL, relative or absolute, the URL
    /// content is replaced with the components of the newly set URL.
    ///
    /// The input `url` must point to a correctly formatted "RFC 3986+" URL.
    /// The URL parser only understands and parses the subset of URLS that are
    /// "hierarchical" and therefore contain a `://` separator - not the ones
    /// that are normally specified with only a colon separator.
    ///
    /// By default this API only parses URLs using schemes for protocols that
    /// are supported built-in. To make libcurl parse URLs generically even for
    /// schemes it does not know about, the
    /// [`SetFlags::non_support_scheme()`] option must be set. Otherwise,
    /// this function returns `UNSUPPORTED_SCHEME` for URL schemes it does not
    /// recognize.
    ///
    /// Unless [`SetFlags::no_authority()`] is set, a blank hostname is not
    /// allowed in the URL.
    ///
    /// When a full URL is set (parsed), the hostname component is stored URL
    /// decoded.
    ///
    /// It is considered fine to set a blank URL (`""`) as a redirect, but not
    /// as a normal URL. Therefore, setting a `""`` URL works fine if the
    /// handle already holds a URL, otherwise it triggers an error.
    pub fn set_url(&mut self, url: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_URL, Some(url), flags)
    }

    /// Return the slightly cleaned up version of full URL using all available parts.
    ///
    /// We advise using the [`GetFlags::punycode()`] option to get the URL as
    /// "normalized" as possible since IDN allows hostnames to be written in
    /// many different ways that still end up the same punycode version.
    ///
    /// Zero-length queries and fragments are excluded from the URL unless
    /// [`GetFlags::get_empty()`] is set.
    pub fn get_url(&self, flags: GetFlags) -> Result<String, Error> {
        let url = self.ffi_get(curl_sys::CURLUPART_URL, flags, curl_sys::CURLUE_OK)?;
        Ok(url.unwrap_or_default())
    }

    /// Get the scheme part of the URL.
    ///
    /// Scheme cannot be URL decoded on set. libcurl only accepts setting
    /// schemes up to 40 bytes long.
    pub fn get_scheme(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(
            curl_sys::CURLUPART_SCHEME,
            flags,
            curl_sys::CURLUE_NO_SCHEME,
        )
    }

    /// Set the scheme part of the URL.
    ///
    /// Scheme cannot be URL decoded on set. libcurl only accepts setting
    /// schemes up to 40 bytes long.
    pub fn set_scheme(&mut self, scheme: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_SCHEME, Some(scheme), flags)
    }

    /// Clear the scheme part of the URL.
    pub fn clear_scheme(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_SCHEME, None, SetFlags::new())
    }

    /// Get the user part of the URL.
    pub fn get_user(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(curl_sys::CURLUPART_USER, flags, curl_sys::CURLUE_NO_USER)
    }

    /// Set the user part of the URL.
    ///
    /// If only the user part is set and not the password, the URL is
    /// represented with a blank password.
    pub fn set_user(&mut self, user: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_USER, Some(user), flags)
    }

    /// Clear the user part of the URL.
    pub fn clear_user(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_USER, None, SetFlags::new())
    }

    /// Get the password part of the URL.
    pub fn get_password(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(
            curl_sys::CURLUPART_PASSWORD,
            flags,
            curl_sys::CURLUE_NO_PASSWORD,
        )
    }

    /// Set the password part of the URL.
    ///
    /// If only the password part is set and not the user, the URL is
    /// represented with a blank user.
    pub fn set_password(&mut self, password: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_PASSWORD, Some(password), flags)
    }

    /// Clear the password part of the URL.
    pub fn clear_password(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_PASSWORD, None, SetFlags::new())
    }

    /// Get the options part of the URL.
    ///
    /// The options field is an optional field that might follow the password
    /// in the userinfo part. It is only recognized/used when parsing URLs for
    /// the following schemes: pop3, smtp and imap. The URL API still allows
    /// users to set and get this field independently of scheme when not
    /// parsing full URLs.
    pub fn get_options(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(
            curl_sys::CURLUPART_OPTIONS,
            flags,
            curl_sys::CURLUE_NO_OPTIONS,
        )
    }

    /// Set the options part of the URL.
    ///
    /// The options field is an optional field that might follow the password
    /// in the userinfo part. It is only recognized/used when parsing URLs for
    /// the following schemes: pop3, smtp and imap. This function however allows
    /// users to independently set this field.
    pub fn set_options(&mut self, options: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_OPTIONS, Some(options), flags)
    }

    /// Clear the options part of the URL.
    pub fn clear_options(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_OPTIONS, None, SetFlags::new())
    }

    /// Get the host part of the URL.
    ///
    /// If it is an IPv6 numeric address, the zone id is not part of it but is provided separately in CURLUPART_ZONEID. IPv6 numerical addresses are returned within brackets ([]).
    ///
    /// IPv6 names are normalized when set, which should make them as short as possible while maintaining correct syntax.
    pub fn get_host(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(curl_sys::CURLUPART_HOST, flags, curl_sys::CURLUE_NO_HOST)
    }

    /// Set the host part of the URL.
    ///
    /// If it is International Domain Name (IDN) the string must then be
    /// encoded as your locale says or UTF-8 (when WinIDN is used). If it is a
    /// bracketed IPv6 numeric address it may contain a zone id (or you can use
    /// [`Url::set_zone_id()`]).
    ///
    /// Note that if you set an IPv6 address, it gets ruined and causes an
    /// error if you also set [`SetFlags::urlencode()`].
    ///
    /// Unless [`SetFlags::no_authority()`] is set, a blank hostname is not
    /// allowed to set.
    pub fn set_host(&mut self, host: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_HOST, Some(host), flags)
    }

    /// Clear the host part of the URL.
    pub fn clear_host(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_HOST, None, SetFlags::new())
    }

    /// Get the zone id part of the URL.
    ///
    /// If the hostname is a numeric IPv6 address, this field might also be set.
    pub fn get_zone_id(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(
            curl_sys::CURLUPART_ZONEID,
            flags,
            curl_sys::CURLUE_NO_ZONEID,
        )
    }

    /// Set the zone id part of the URL.
    ///
    /// If the hostname is a numeric IPv6 address, this field can also be set.
    pub fn set_zone_id(&mut self, zone_id: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_ZONEID, Some(zone_id), flags)
    }

    /// Clear the zone id part of the URL.
    pub fn clear_zone_id(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_ZONEID, None, SetFlags::new())
    }

    /// Get the port part of the URL.
    ///
    /// A port cannot be URL decoded on get.
    pub fn get_port(&self, flags: GetFlags) -> Result<Option<u16>, Error> {
        let port = self.ffi_get(curl_sys::CURLUPART_PORT, flags, curl_sys::CURLUE_NO_PORT)?;
        port.map(|s| {
            s.parse::<u16>()
                .map_err(|_| Error::new(curl_sys::CURLUE_MALFORMED_INPUT))
        })
        .transpose()
    }

    /// Set the port part of the URL.
    ///
    /// The port number cannot be URL encoded on set.
    pub fn set_port(&mut self, port: u16, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_PORT, Some(&port.to_string()), flags)
    }

    /// Clear the port part of the URL.
    pub fn clear_port(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_PORT, None, SetFlags::new())
    }

    /// Get the path part of the URL.
    ///
    /// The part is always at least a slash ('/') even if no path was supplied
    /// in the URL. A URL path always starts with a slash.
    pub fn get_path(&self, flags: GetFlags) -> Result<String, Error> {
        let path = self.ffi_get(curl_sys::CURLUPART_PATH, flags, curl_sys::CURLUE_OK)?;
        Ok(path.unwrap_or_else(|| "/".to_string()))
    }

    /// Set the path part of the URL.
    ///
    /// If a path is set in the URL without a leading slash, a slash is prepended automatically.
    pub fn set_path(&mut self, path: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_PATH, Some(path), flags)
    }

    /// Get the query part of the URL.
    ///
    /// The initial question mark that denotes the beginning of the query part
    /// is a delimiter only. It is not part of the query contents.
    ///
    /// A not-present query returns [`None`].
    ///
    /// A zero-length query returns part as [`None`] unless
    /// [`GetFlags::get_empty()`] is set.
    ///
    /// The query part gets pluses converted to space when asked to URL decode
    /// on get with [`GetFlags::urldecode()`] set.
    pub fn get_query(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(curl_sys::CURLUPART_QUERY, flags, curl_sys::CURLUE_NO_QUERY)
    }

    /// Set the query part of the URL.
    ///
    /// The query part gets spaces converted to pluses when asked to URL encode
    /// on set with [`SetFlags::urlencode()`].
    ///
    /// If used together with [`SetFlags::append_query()`], the provided part is
    /// appended on the end of the existing query.
    ///
    /// The question mark in the URL is not part of the actual query contents.
    pub fn set_query(&mut self, query: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_QUERY, Some(query), flags)
    }

    /// Clear the query part of the URL.
    pub fn clear_query(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_QUERY, None, SetFlags::new())
    }

    /// Get the fragment part of the URL.
    ///
    /// The initial hash sign that denotes the beginning of the fragment is
    /// a delimiter only. It is not part of the fragment contents.
    ///
    /// A not-present fragment returns part set to None.
    ///
    /// A zero-length fragment returns part as NULL unless CURLU_GET_EMPTY is set.
    pub fn get_fragment(&self, flags: GetFlags) -> Result<Option<String>, Error> {
        self.ffi_get(
            curl_sys::CURLUPART_FRAGMENT,
            flags,
            curl_sys::CURLUE_NO_FRAGMENT,
        )
    }

    /// Set the fragment part of the URL.
    ///
    /// The hash sign in the URL is not part of the actual fragment contents.
    pub fn set_fragment(&mut self, fragment: &str, flags: SetFlags) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_FRAGMENT, Some(fragment), flags)
    }

    /// Clear the fragment part of the URL.
    pub fn clear_fragment(&mut self) -> Result<(), Error> {
        self.ffi_set(curl_sys::CURLUPART_FRAGMENT, None, SetFlags::new())
    }
}

impl Clone for Url {
    fn clone(&self) -> Self {
        let new_handle = unsafe { curl_sys::curl_url_dup(self.raw) };
        Self { raw: new_handle }
    }
}

impl Drop for Url {
    fn drop(&mut self) {
        unsafe {
            curl_sys::curl_url_cleanup(self.raw);
        }
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_url(GetFlags::new()) {
            Ok(url) => url.fmt(f),
            Err(_) => "".fmt(f),
        }
    }
}

impl fmt::Debug for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Url").field("raw", &self.raw).finish()
    }
}

fn cvt(code: curl_sys::CURLUcode) -> Result<(), Error> {
    if code == curl_sys::CURLUE_OK {
        Ok(())
    } else {
        Err(Error::new(code))
    }
}
