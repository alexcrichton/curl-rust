use std::ffi::c_uint;
use std::fmt;

/// A set of options that can be used with `Url::get*` methods to modify the
/// behavior of URL retrieval.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct GetFlags(pub(super) c_uint);

impl GetFlags {
    /// Creates a new [`GetFlags`] instance with no options set.
    pub const fn new() -> Self {
        Self(0)
    }

    /// If the handle has no port stored, this option makes curl_url_get return
    /// the default port for the used scheme.
    pub const fn default_port(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_DEFAULT_PORT)
    }

    /// If the handle has no scheme stored, this option makes curl_url_get
    /// return the default scheme instead of error.
    pub const fn default_scheme(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_DEFAULT_SCHEME)
    }

    /// Instructs libcurl to not return a port number if it matches the
    /// default port for the scheme.
    pub const fn no_default_port(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_NO_DEFAULT_PORT)
    }

    /// Asks libcurl to URL decode the contents before returning it. It does
    /// not decode the scheme, the port number or the full URL.
    ///
    /// The query component also gets plus-to-space conversion as a bonus when
    /// this bit is set.
    ///
    /// Note that this URL decoding is charset unaware and you get a zero
    /// terminated string back with data that could be intended for a
    /// particular encoding.
    ///
    /// If there are byte values lower than 32 in the decoded string, the get
    /// operation returns an error instead.
    pub const fn urldecode(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_URLDECODE)
    }

    /// If set, libcurl encodes the hostname part when a full URL is retrieved.
    /// If not set (default), libcurl returns the URL with the hostname raw to
    /// support IDN names to appear as-is. IDN hostnames are typically using
    /// non-ASCII bytes that otherwise gets percent-encoded.
    ///
    /// Note that even when not asking for URL encoding, the '%' (byte 37) is
    /// URL encoded to make sure the hostname remains valid.
    pub const fn urlencode(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_URLENCODE)
    }

    /// If set and [`GetFlags::urlencode()`] is not set, and asked to
    /// retrieve the host or full URL parts, libcurl returns the host name in
    /// its punycode version if it contains any non-ASCII octets (and is an
    /// IDN name).
    ///
    /// If libcurl is built without IDN capabilities, using this bit makes
    /// curl_url_get return `LACKS_IDN` if the hostname contains anything
    /// outside the ASCII range.
    ///
    /// Added in curl 7.88.0.
    pub const fn punycode(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_PUNYCODE)
    }

    /// If set and asked to retrieve the CURLUPART_HOST or CURLUPART_URL parts,
    /// libcurl returns the hostname in its IDN (International Domain Name)
    /// UTF-8 version if it otherwise is a punycode version. If the punycode
    /// name cannot be converted to IDN correctly, libcurl returns
    /// `CURLUE_BAD_HOSTNAME`.
    ///
    /// If libcurl is built without IDN capabilities, using this bit makes
    /// libcurl return `CURLUE_LACKS_IDN` if the hostname is using punycode.
    ///
    /// Added in curl 8.3.0
    pub const fn punycode2idn(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_PUNY2IDN)
    }

    /// When this flag is used, it makes the function return empty query and
    /// fragments parts or when used in the full URL. By default, libcurl
    /// otherwise considers empty parts non-existing.
    ///
    /// An empty query part is one where this is nothing following the question
    /// mark (before the possible fragment). An empty fragments part is one
    /// where there is nothing following the hash sign.
    ///
    /// Added in curl 8.8.0
    pub const fn get_empty(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_GET_EMPTY)
    }

    /// When this flag is used, it treats the scheme as non-existing if it was
    /// set as a result of a previous guess; when `guess_scheme` was used
    /// parsing a URL.
    ///
    /// Using this flag when getting scheme if the scheme was set as the result
    /// of a guess makes libcurl return `CURLUE_NO_SCHEME`.
    ///
    /// Using this flag when getting full URL if the scheme was set as the
    /// result of a guess makes libcurl return the full URL without the scheme
    /// component. Such a URL can then only be parsed with `set_url` if
    /// `CURLU_GUESS_SCHEME` is used.
    ///
    /// Added in curl 8.9.0
    pub const fn no_guess_scheme(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_NO_GUESS_SCHEME)
    }
}

impl fmt::Debug for GetFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetFlags")
            .field("value", &format!("{:032b}", self.0).as_str())
            .finish()
    }
}
