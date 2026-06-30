use std::ffi::c_uint;
use std::fmt;

/// A set of options that can be used with `Url::get*` methods to modify the
/// behavior of URL retrieval.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct SetFlags(pub(super) c_uint);

impl SetFlags {
    /// Creates a new [`SetFlags`] instance with no options set.
    pub const fn new() -> Self {
        Self(0)
    }

    /// If the handle has no port stored, this option makes curl_url_get return
    /// the default port for the used scheme.
    pub const fn append_query(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_APPENDQUERY)
    }

    /// If set, allows libcurl to set a non-supported scheme. It then of course
    /// course cannot know if the provided scheme is a valid one or not.
    pub const fn non_support_scheme(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_NON_SUPPORT_SCHEME)
    }

    /// When set, libcurl URL encodes the part on entry, except for scheme, port
    /// and URL.
    ///
    /// When setting the path component with URL encoding enabled, the slash
    /// character is skipped.
    ///
    /// The query part gets space-to-plus converted before the URL conversion is
    /// applied.
    ///
    /// This URL encoding is charset unaware and converts the input in a
    /// byte-by-byte manner.
    pub const fn urlencode(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_URLENCODE)
    }

    /// If the handle has no scheme stored, this option makes curl_url_get
    /// return the default scheme instead of error.
    pub const fn default_scheme(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_DEFAULT_SCHEME)
    }

    /// If set, allows the URL to be set without a scheme and it instead
    /// "guesses" which scheme that was intended based on the hostname. If the
    /// outermost subdomain name matches DICT, FTP, IMAP, LDAP, POP3 or SMTP
    /// then that scheme is used, otherwise it picks HTTP. Conflicts with the
    /// [`SetFlags::default_scheme`] option which takes precedence if both are
    /// set.
    ///
    /// If guessing is not allowed and there is no default scheme set, trying
    /// to parse a URL without a scheme returns error.
    ///
    /// If the scheme ends up set as a result of guessing, i.e. it is not
    /// actually present in the parsed URL, it can later be figured out by
    /// using the `no_guess_scheme` flag when subsequently getting the URL or
    /// the scheme.
    pub const fn guess_scheme(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_GUESS_SCHEME)
    }

    /// If set, skips authority checks. The RFC allows individual schemes to
    /// omit the host part (normally the only mandatory part of the authority),
    /// but libcurl cannot know whether this is permitted for custom schemes.
    /// Specifying the flag permits empty authority sections, similar to how
    /// file scheme is handled.
    pub const fn no_authority(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_NO_AUTHORITY)
    }

    /// When set for full URL, this skips the normalization of the path. That is
    /// the procedure where libcurl otherwise removes sequences of dot-slash and
    /// dot-dot etc. The same option used for transfers is called
    /// `CURLOPT_PATH_AS_IS`.
    pub const fn path_as_is(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_PATH_AS_IS)
    }

    /// If set, the URL parser allows space (ASCII 32) where possible. The URL
    /// syntax does normally not allow spaces anywhere, but they should be
    /// encoded as %20 or '+'. When spaces are allowed, they are still not
    /// allowed in the scheme. When space is used and allowed in a URL, it is
    /// stored as-is unless CURLU_URLENCODE is also set, which then makes
    /// libcurl URL encode the space before stored. This affects how the URL is
    /// constructed when curl_url_get is subsequently used to extract the full
    /// URL or individual parts.
    ///
    /// Added in 7.78.0
    pub const fn allow_space(&self) -> Self {
        Self(self.0 | curl_sys::CURLU_ALLOW_SPACE)
    }
}

impl fmt::Debug for SetFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SetFlags")
            .field("value", &format!("{:032b}", self.0).as_str())
            .finish()
    }
}
