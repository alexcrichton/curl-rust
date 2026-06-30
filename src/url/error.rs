use std::{
    fmt::{self},
    io,
};

/// Represents an error that can occur when working with URLs in libcurl.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error {
    code: curl_sys::CURLUcode,
}

impl Error {
    /// Creates a new `Error` from a `CURLUcode`.
    pub fn new(code: curl_sys::CURLUcode) -> Self {
        Self { code }
    }

    /// Returns the underlying `CURLUcode` of the error.
    pub fn code(&self) -> curl_sys::CURLUcode {
        self.code
    }

    /// Returns a string representation of the error.
    pub fn description(&self) -> &'static str {
        use curl_sys::*;

        match self.code {
            CURLUE_OK => "No error",
            CURLUE_BAD_HANDLE => "An invalid CURLU pointer was passed as argument",
            CURLUE_BAD_PARTPOINTER => "An invalid 'part' argument was passed as argument",
            CURLUE_MALFORMED_INPUT => "Malformed input to a URL function",
            CURLUE_BAD_PORT_NUMBER => "Port number was not a decimal number between 0 and 65535",
            CURLUE_UNSUPPORTED_SCHEME => "Unsupported URL scheme",
            CURLUE_URLDECODE => "URL decode error, most likely because of rubbish in the input",
            CURLUE_OUT_OF_MEMORY => "A memory function failed",
            CURLUE_USER_NOT_ALLOWED => "Credentials was passed in the URL when prohibited",
            CURLUE_UNKNOWN_PART => "An unknown part ID was passed to a URL API function",
            CURLUE_NO_SCHEME => "No scheme part in the URL",
            CURLUE_NO_USER => "No user part in the URL",
            CURLUE_NO_PASSWORD => "No password part in the URL",
            CURLUE_NO_OPTIONS => "No options part in the URL",
            CURLUE_NO_HOST => "No host part in the URL",
            CURLUE_NO_PORT => "No port part in the URL",
            CURLUE_NO_QUERY => "No query part in the URL",
            CURLUE_NO_FRAGMENT => "No fragment part in the URL",
            CURLUE_NO_ZONEID => "No zoneid part in the URL",
            CURLUE_BAD_LOGIN => "Bad login part",
            CURLUE_BAD_IPV6 => "Bad IPv6 address",
            CURLUE_BAD_HOSTNAME => "Bad hostname",
            CURLUE_BAD_FILE_URL => "Bad file:// URL",
            CURLUE_BAD_SLASHES => "Unsupported number of slashes following scheme",
            CURLUE_BAD_SCHEME => "Bad scheme",
            CURLUE_BAD_PATH => "Bad path",
            CURLUE_BAD_FRAGMENT => "Bad fragment",
            CURLUE_BAD_QUERY => "Bad query",
            CURLUE_BAD_PASSWORD => "Bad password",
            CURLUE_BAD_USER => "Bad user",
            CURLUE_LACKS_IDN => "libcurl lacks IDN support",
            CURLUE_TOO_LARGE => "A value or data field is larger than allowed",
            _ => "Error",
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("code", &self.code)
            .field("description", &self.description())
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, err.description())
    }
}
