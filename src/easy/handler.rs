use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, SeekFrom, Write};
use std::path::Path;
use std::slice;
use std::str;
use std::time::Duration;

use curl_sys;
use libc::{self, c_char, c_double, c_int, c_long, c_ulong, c_void, size_t};
use socket2::Socket;

use easy::form;
use easy::list;
use easy::windows;
use easy::{Form, List};
use panic;
use Error;

/// A trait for the various callbacks used by libcurl to invoke user code.
///
/// This trait represents all operations that libcurl can possibly invoke a
/// client for code during an HTTP transaction. Each callback has a default
/// "noop" implementation, the same as in libcurl. Types implementing this trait
/// may simply override the relevant functions to learn about the callbacks
/// they're interested in.
///
/// # Examples
///
/// ```
/// use curl::easy::{Easy2, Handler, WriteError};
///
/// struct Collector(Vec<u8>);
///
/// impl Handler for Collector {
///     fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
///         self.0.extend_from_slice(data);
///         Ok(data.len())
///     }
/// }
///
/// let mut easy = Easy2::new(Collector(Vec::new()));
/// easy.get(true).unwrap();
/// easy.url("https://www.rust-lang.org/").unwrap();
/// easy.perform().unwrap();
///
/// assert_eq!(easy.response_code().unwrap(), 200);
/// let contents = easy.get_ref();
/// println!("{}", String::from_utf8_lossy(&contents.0));
/// ```
pub trait Handler {
    /// Callback invoked whenever curl has downloaded data for the application.
    ///
    /// This callback function gets called by libcurl as soon as there is data
    /// received that needs to be saved.
    ///
    /// The callback function will be passed as much data as possible in all
    /// invokes, but you must not make any assumptions. It may be one byte, it
    /// may be thousands. If `show_header` is enabled, which makes header data
    /// get passed to the write callback, you can get up to
    /// `CURL_MAX_HTTP_HEADER` bytes of header data passed into it.  This
    /// usually means 100K.
    ///
    /// This function may be called with zero bytes data if the transferred file
    /// is empty.
    ///
    /// The callback should return the number of bytes actually taken care of.
    /// If that amount differs from the amount passed to your callback function,
    /// it'll signal an error condition to the library. This will cause the
    /// transfer to get aborted and the libcurl function used will return
    /// an error with `is_write_error`.
    ///
    /// If your callback function returns `Err(WriteError::Pause)` it will cause
    /// this transfer to become paused. See `unpause_write` for further details.
    ///
    /// By default data is sent into the void, and this corresponds to the
    /// `CURLOPT_WRITEFUNCTION` and `CURLOPT_WRITEDATA` options.
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        Ok(data.len())
    }

    /// Read callback for data uploads.
    ///
    /// This callback function gets called by libcurl as soon as it needs to
    /// read data in order to send it to the peer - like if you ask it to upload
    /// or post data to the server.
    ///
    /// Your function must then return the actual number of bytes that it stored
    /// in that memory area. Returning 0 will signal end-of-file to the library
    /// and cause it to stop the current transfer.
    ///
    /// If you stop the current transfer by returning 0 "pre-maturely" (i.e
    /// before the server expected it, like when you've said you will upload N
    /// bytes and you upload less than N bytes), you may experience that the
    /// server "hangs" waiting for the rest of the data that won't come.
    ///
    /// The read callback may return `Err(ReadError::Abort)` to stop the
    /// current operation immediately, resulting in a `is_aborted_by_callback`
    /// error code from the transfer.
    ///
    /// The callback can return `Err(ReadError::Pause)` to cause reading from
    /// this connection to pause. See `unpause_read` for further details.
    ///
    /// By default data not input, and this corresponds to the
    /// `CURLOPT_READFUNCTION` and `CURLOPT_READDATA` options.
    ///
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `read_function` to configure a
    /// callback that can reference stack-local data.
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        drop(data);
        Ok(0)
    }

    /// User callback for seeking in input stream.
    ///
    /// This function gets called by libcurl to seek to a certain position in
    /// the input stream and can be used to fast forward a file in a resumed
    /// upload (instead of reading all uploaded bytes with the normal read
    /// function/callback). It is also called to rewind a stream when data has
    /// already been sent to the server and needs to be sent again. This may
    /// happen when doing a HTTP PUT or POST with a multi-pass authentication
    /// method, or when an existing HTTP connection is reused too late and the
    /// server closes the connection.
    ///
    /// The callback function must return `SeekResult::Ok` on success,
    /// `SeekResult::Fail` to cause the upload operation to fail or
    /// `SeekResult::CantSeek` to indicate that while the seek failed, libcurl
    /// is free to work around the problem if possible. The latter can sometimes
    /// be done by instead reading from the input or similar.
    ///
    /// By default data this option is not set, and this corresponds to the
    /// `CURLOPT_SEEKFUNCTION` and `CURLOPT_SEEKDATA` options.
    fn seek(&mut self, whence: SeekFrom) -> SeekResult {
        drop(whence);
        SeekResult::CantSeek
    }

    /// Specify a debug callback
    ///
    /// `debug_function` replaces the standard debug function used when
    /// `verbose` is in effect. This callback receives debug information,
    /// as specified in the type argument.
    ///
    /// By default this option is not set and corresponds to the
    /// `CURLOPT_DEBUGFUNCTION` and `CURLOPT_DEBUGDATA` options.
    fn debug(&mut self, kind: InfoType, data: &[u8]) {
        debug(kind, data)
    }

    /// Callback that receives header data
    ///
    /// This function gets called by libcurl as soon as it has received header
    /// data. The header callback will be called once for each header and only
    /// complete header lines are passed on to the callback. Parsing headers is
    /// very easy using this. If this callback returns `false` it'll signal an
    /// error to the library. This will cause the transfer to get aborted and
    /// the libcurl function in progress will return `is_write_error`.
    ///
    /// A complete HTTP header that is passed to this function can be up to
    /// CURL_MAX_HTTP_HEADER (100K) bytes.
    ///
    /// It's important to note that the callback will be invoked for the headers
    /// of all responses received after initiating a request and not just the
    /// final response. This includes all responses which occur during
    /// authentication negotiation. If you need to operate on only the headers
    /// from the final response, you will need to collect headers in the
    /// callback yourself and use HTTP status lines, for example, to delimit
    /// response boundaries.
    ///
    /// When a server sends a chunked encoded transfer, it may contain a
    /// trailer. That trailer is identical to a HTTP header and if such a
    /// trailer is received it is passed to the application using this callback
    /// as well. There are several ways to detect it being a trailer and not an
    /// ordinary header: 1) it comes after the response-body. 2) it comes after
    /// the final header line (CR LF) 3) a Trailer: header among the regular
    /// response-headers mention what header(s) to expect in the trailer.
    ///
    /// For non-HTTP protocols like FTP, POP3, IMAP and SMTP this function will
    /// get called with the server responses to the commands that libcurl sends.
    ///
    /// By default this option is not set and corresponds to the
    /// `CURLOPT_HEADERFUNCTION` and `CURLOPT_HEADERDATA` options.
    fn header(&mut self, data: &[u8]) -> bool {
        drop(data);
        true
    }

    /// Callback to progress meter function
    ///
    /// This function gets called by libcurl instead of its internal equivalent
    /// with a frequent interval. While data is being transferred it will be
    /// called very frequently, and during slow periods like when nothing is
    /// being transferred it can slow down to about one call per second.
    ///
    /// The callback gets told how much data libcurl will transfer and has
    /// transferred, in number of bytes. The first argument is the total number
    /// of bytes libcurl expects to download in this transfer. The second
    /// argument is the number of bytes downloaded so far. The third argument is
    /// the total number of bytes libcurl expects to upload in this transfer.
    /// The fourth argument is the number of bytes uploaded so far.
    ///
    /// Unknown/unused argument values passed to the callback will be set to
    /// zero (like if you only download data, the upload size will remain 0).
    /// Many times the callback will be called one or more times first, before
    /// it knows the data sizes so a program must be made to handle that.
    ///
    /// Returning `false` from this callback will cause libcurl to abort the
    /// transfer and return `is_aborted_by_callback`.
    ///
    /// If you transfer data with the multi interface, this function will not be
    /// called during periods of idleness unless you call the appropriate
    /// libcurl function that performs transfers.
    ///
    /// `progress` must be set to `true` to make this function actually get
    /// called.
    ///
    /// By default this function calls an internal method and corresponds to
    /// `CURLOPT_PROGRESSFUNCTION` and `CURLOPT_PROGRESSDATA`.
    fn progress(&mut self, dltotal: f64, dlnow: f64, ultotal: f64, ulnow: f64) -> bool {
        drop((dltotal, dlnow, ultotal, ulnow));
        true
    }

    /// Callback to SSL context
    ///
    /// This callback function gets called by libcurl just before the
    /// initialization of an SSL connection after having processed all
    /// other SSL related options to give a last chance to an
    /// application to modify the behaviour of the SSL
    /// initialization. The `ssl_ctx` parameter is actually a pointer
    /// to the SSL library's SSL_CTX. If an error is returned from the
    /// callback no attempt to establish a connection is made and the
    /// perform operation will return the callback's error code.
    ///
    /// This function will get called on all new connections made to a
    /// server, during the SSL negotiation. The SSL_CTX pointer will
    /// be a new one every time.
    ///
    /// To use this properly, a non-trivial amount of knowledge of
    /// your SSL library is necessary. For example, you can use this
    /// function to call library-specific callbacks to add additional
    /// validation code for certificates, and even to change the
    /// actual URI of a HTTPS request.
    ///
    /// By default this function calls an internal method and
    /// corresponds to `CURLOPT_SSL_CTX_FUNCTION` and
    /// `CURLOPT_SSL_CTX_DATA`.
    ///
    /// Note that this callback is not guaranteed to be called, not all versions
    /// of libcurl support calling this callback.
    fn ssl_ctx(&mut self, cx: *mut c_void) -> Result<(), Error> {
        // By default, if we're on an OpenSSL enabled libcurl and we're on
        // Windows, add the system's certificate store to OpenSSL's certificate
        // store.
        ssl_ctx(cx)
    }

    /// Callback to open sockets for libcurl.
    ///
    /// This callback function gets called by libcurl instead of the socket(2)
    /// call. The callback function should return the newly created socket
    /// or `None` in case no connection could be established or another
    /// error was detected. Any additional `setsockopt(2)` calls can of course
    /// be done on the socket at the user's discretion. A `None` return
    /// value from the callback function will signal an unrecoverable error to
    /// libcurl and it will return `is_couldnt_connect` from the function that
    /// triggered this callback.
    ///
    /// By default this function opens a standard socket and
    /// corresponds to `CURLOPT_OPENSOCKETFUNCTION `.
    fn open_socket(
        &mut self,
        family: c_int,
        socktype: c_int,
        protocol: c_int,
    ) -> Option<curl_sys::curl_socket_t> {
        // Note that we override this to calling a function in `socket2` to
        // ensure that we open all sockets with CLOEXEC. Otherwise if we rely on
        // libcurl to open sockets it won't use CLOEXEC.
        return Socket::new(family.into(), socktype.into(), Some(protocol.into()))
            .ok()
            .map(cvt);

        #[cfg(unix)]
        fn cvt(socket: Socket) -> curl_sys::curl_socket_t {
            use std::os::unix::prelude::*;
            socket.into_raw_fd()
        }

        #[cfg(windows)]
        fn cvt(socket: Socket) -> curl_sys::curl_socket_t {
            use std::os::windows::prelude::*;
            socket.into_raw_socket()
        }
    }
}

pub fn debug(kind: InfoType, data: &[u8]) {
    let out = io::stderr();
    let prefix = match kind {
        InfoType::Text => "*",
        InfoType::HeaderIn => "<",
        InfoType::HeaderOut => ">",
        InfoType::DataIn | InfoType::SslDataIn => "{",
        InfoType::DataOut | InfoType::SslDataOut => "}",
        InfoType::__Nonexhaustive => " ",
    };
    let mut out = out.lock();
    drop(write!(out, "{} ", prefix));
    match str::from_utf8(data) {
        Ok(s) => drop(out.write_all(s.as_bytes())),
        Err(_) => drop(write!(out, "({} bytes of data)\n", data.len())),
    }
}

pub fn ssl_ctx(cx: *mut c_void) -> Result<(), Error> {
    windows::add_certs_to_context(cx);
    Ok(())
}

/// Raw bindings to a libcurl "easy session".
///
/// This type corresponds to the `CURL` type in libcurl, and is probably what
/// you want for just sending off a simple HTTP request and fetching a response.
/// Each easy handle can be thought of as a large builder before calling the
/// final `perform` function.
///
/// There are many many configuration options for each `Easy2` handle, and they
/// should all have their own documentation indicating what it affects and how
/// it interacts with other options. Some implementations of libcurl can use
/// this handle to interact with many different protocols, although by default
/// this crate only guarantees the HTTP/HTTPS protocols working.
///
/// Note that almost all methods on this structure which configure various
/// properties return a `Result`. This is largely used to detect whether the
/// underlying implementation of libcurl actually implements the option being
/// requested. If you're linked to a version of libcurl which doesn't support
/// the option, then an error will be returned. Some options also perform some
/// validation when they're set, and the error is returned through this vector.
///
/// Note that historically this library contained an `Easy` handle so this one's
/// called `Easy2`. The major difference between the `Easy` type is that an
/// `Easy2` structure uses a trait instead of closures for all of the callbacks
/// that curl can invoke. The `Easy` type is actually built on top of this
/// `Easy` type, and this `Easy2` type can be more flexible in some situations
/// due to the generic parameter.
///
/// There's not necessarily a right answer for which type is correct to use, but
/// as a general rule of thumb `Easy` is typically a reasonable choice for
/// synchronous I/O and `Easy2` is a good choice for asynchronous I/O.
///
/// # Examples
///
/// ```
/// use curl::easy::{Easy2, Handler, WriteError};
///
/// struct Collector(Vec<u8>);
///
/// impl Handler for Collector {
///     fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
///         self.0.extend_from_slice(data);
///         Ok(data.len())
///     }
/// }
///
/// let mut easy = Easy2::new(Collector(Vec::new()));
/// easy.get(true).unwrap();
/// easy.url("https://www.rust-lang.org/").unwrap();
/// easy.perform().unwrap();
///
/// assert_eq!(easy.response_code().unwrap(), 200);
/// let contents = easy.get_ref();
/// println!("{}", String::from_utf8_lossy(&contents.0));
/// ```
pub struct Easy2<H> {
    inner: Box<Inner<H>>,
}

struct Inner<H> {
    handle: *mut curl_sys::CURL,
    header_list: Option<List>,
    resolve_list: Option<List>,
    form: Option<Form>,
    error_buf: RefCell<Vec<u8>>,
    handler: H,
}

unsafe impl<H: Send> Send for Inner<H> {}

/// Possible proxy types that libcurl currently understands.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum ProxyType {
    Http = curl_sys::CURLPROXY_HTTP as isize,
    Http1 = curl_sys::CURLPROXY_HTTP_1_0 as isize,
    Socks4 = curl_sys::CURLPROXY_SOCKS4 as isize,
    Socks5 = curl_sys::CURLPROXY_SOCKS5 as isize,
    Socks4a = curl_sys::CURLPROXY_SOCKS4A as isize,
    Socks5Hostname = curl_sys::CURLPROXY_SOCKS5_HOSTNAME as isize,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Possible conditions for the `time_condition` method.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum TimeCondition {
    None = curl_sys::CURL_TIMECOND_NONE as isize,
    IfModifiedSince = curl_sys::CURL_TIMECOND_IFMODSINCE as isize,
    IfUnmodifiedSince = curl_sys::CURL_TIMECOND_IFUNMODSINCE as isize,
    LastModified = curl_sys::CURL_TIMECOND_LASTMOD as isize,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Possible values to pass to the `ip_resolve` method.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum IpResolve {
    V4 = curl_sys::CURL_IPRESOLVE_V4 as isize,
    V6 = curl_sys::CURL_IPRESOLVE_V6 as isize,
    Any = curl_sys::CURL_IPRESOLVE_WHATEVER as isize,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive = 500,
}

/// Possible values to pass to the `http_version` method.
#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
    /// We don't care what http version to use, and we'd like the library to
    /// choose the best possible for us.
    Any = curl_sys::CURL_HTTP_VERSION_NONE as isize,

    /// Please use HTTP 1.0 in the request
    V10 = curl_sys::CURL_HTTP_VERSION_1_0 as isize,

    /// Please use HTTP 1.1 in the request
    V11 = curl_sys::CURL_HTTP_VERSION_1_1 as isize,

    /// Please use HTTP 2 in the request
    /// (Added in CURL 7.33.0)
    V2 = curl_sys::CURL_HTTP_VERSION_2_0 as isize,

    /// Use version 2 for HTTPS, version 1.1 for HTTP
    /// (Added in CURL 7.47.0)
    V2TLS = curl_sys::CURL_HTTP_VERSION_2TLS as isize,

    /// Please use HTTP 2 without HTTP/1.1 Upgrade
    /// (Added in CURL 7.49.0)
    V2PriorKnowledge = curl_sys::CURL_HTTP_VERSION_2_PRIOR_KNOWLEDGE as isize,

    /// Setting this value will make libcurl attempt to use HTTP/3 directly to
    /// server given in the URL. Note that this cannot gracefully downgrade to
    /// earlier HTTP version if the server doesn't support HTTP/3.
    ///
    /// For more reliably upgrading to HTTP/3, set the preferred version to
    /// something lower and let the server announce its HTTP/3 support via
    /// Alt-Svc:.
    ///
    /// (Added in CURL 7.66.0)
    V3 = curl_sys::CURL_HTTP_VERSION_3 as isize,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive = 500,
}

/// Possible values to pass to the `ssl_version` and `ssl_min_max_version` method.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum SslVersion {
    Default = curl_sys::CURL_SSLVERSION_DEFAULT as isize,
    Tlsv1 = curl_sys::CURL_SSLVERSION_TLSv1 as isize,
    Sslv2 = curl_sys::CURL_SSLVERSION_SSLv2 as isize,
    Sslv3 = curl_sys::CURL_SSLVERSION_SSLv3 as isize,
    Tlsv10 = curl_sys::CURL_SSLVERSION_TLSv1_0 as isize,
    Tlsv11 = curl_sys::CURL_SSLVERSION_TLSv1_1 as isize,
    Tlsv12 = curl_sys::CURL_SSLVERSION_TLSv1_2 as isize,
    Tlsv13 = curl_sys::CURL_SSLVERSION_TLSv1_3 as isize,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive = 500,
}

/// Possible return values from the `seek_function` callback.
#[derive(Debug, Clone, Copy)]
pub enum SeekResult {
    /// Indicates that the seek operation was a success
    Ok = curl_sys::CURL_SEEKFUNC_OK as isize,

    /// Indicates that the seek operation failed, and the entire request should
    /// fail as a result.
    Fail = curl_sys::CURL_SEEKFUNC_FAIL as isize,

    /// Indicates that although the seek failed libcurl should attempt to keep
    /// working if possible (for example "seek" through reading).
    CantSeek = curl_sys::CURL_SEEKFUNC_CANTSEEK as isize,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive = 500,
}

/// Possible data chunks that can be witnessed as part of the `debug_function`
/// callback.
#[derive(Debug, Clone, Copy)]
pub enum InfoType {
    /// The data is informational text.
    Text,

    /// The data is header (or header-like) data received from the peer.
    HeaderIn,

    /// The data is header (or header-like) data sent to the peer.
    HeaderOut,

    /// The data is protocol data received from the peer.
    DataIn,

    /// The data is protocol data sent to the peer.
    DataOut,

    /// The data is SSL/TLS (binary) data received from the peer.
    SslDataIn,

    /// The data is SSL/TLS (binary) data sent to the peer.
    SslDataOut,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Possible error codes that can be returned from the `read_function` callback.
#[derive(Debug)]
pub enum ReadError {
    /// Indicates that the connection should be aborted immediately
    Abort,

    /// Indicates that reading should be paused until `unpause` is called.
    Pause,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Possible error codes that can be returned from the `write_function` callback.
#[derive(Debug)]
pub enum WriteError {
    /// Indicates that reading should be paused until `unpause` is called.
    Pause,

    /// Hidden variant to indicate that this enum should not be matched on, it
    /// may grow over time.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Options for `.netrc` parsing.
#[derive(Debug, Clone, Copy)]
pub enum NetRc {
    /// Ignoring `.netrc` file and use information from url
    ///
    /// This option is default
    Ignored = curl_sys::CURL_NETRC_IGNORED as isize,

    /// The  use of your `~/.netrc` file is optional, and information in the URL is to be
    /// preferred. The file will be scanned for the host and user name (to find the password only)
    /// or for the host only, to find the first user name and password after that machine, which
    /// ever information is not specified in the URL.
    Optional = curl_sys::CURL_NETRC_OPTIONAL as isize,

    /// This value tells the library that use of the file is required, to ignore the information in
    /// the URL, and to search the file for the host only.
    Required = curl_sys::CURL_NETRC_REQUIRED as isize,
}

/// Structure which stores possible authentication methods to get passed to
/// `http_auth` and `proxy_auth`.
#[derive(Clone)]
pub struct Auth {
    bits: c_long,
}

/// Structure which stores possible ssl options to pass to `ssl_options`.
#[derive(Clone)]
pub struct SslOpt {
    bits: c_long,
}

impl<H: Handler> Easy2<H> {
    /// Creates a new "easy" handle which is the core of almost all operations
    /// in libcurl.
    ///
    /// To use a handle, applications typically configure a number of options
    /// followed by a call to `perform`. Options are preserved across calls to
    /// `perform` and need to be reset manually (or via the `reset` method) if
    /// this is not desired.
    pub fn new(handler: H) -> Easy2<H> {
        ::init();
        unsafe {
            let handle = curl_sys::curl_easy_init();
            assert!(!handle.is_null());
            let mut ret = Easy2 {
                inner: Box::new(Inner {
                    handle: handle,
                    header_list: None,
                    resolve_list: None,
                    form: None,
                    error_buf: RefCell::new(vec![0; curl_sys::CURL_ERROR_SIZE]),
                    handler: handler,
                }),
            };
            ret.default_configure();
            return ret;
        }
    }

    /// Re-initializes this handle to the default values.
    ///
    /// This puts the handle to the same state as it was in when it was just
    /// created. This does, however, keep live connections, the session id
    /// cache, the dns cache, and cookies.
    pub fn reset(&mut self) {
        unsafe {
            curl_sys::curl_easy_reset(self.inner.handle);
        }
        self.default_configure();
    }

    fn default_configure(&mut self) {
        self.setopt_ptr(
            curl_sys::CURLOPT_ERRORBUFFER,
            self.inner.error_buf.borrow().as_ptr() as *const _,
        )
        .expect("failed to set error buffer");
        let _ = self.signal(false);
        self.ssl_configure();

        let ptr = &*self.inner as *const _ as *const _;

        let cb: extern "C" fn(*mut c_char, size_t, size_t, *mut c_void) -> size_t = header_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_HEADERFUNCTION, cb as *const _)
            .expect("failed to set header callback");
        self.setopt_ptr(curl_sys::CURLOPT_HEADERDATA, ptr)
            .expect("failed to set header callback");

        let cb: curl_sys::curl_write_callback = write_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_WRITEFUNCTION, cb as *const _)
            .expect("failed to set write callback");
        self.setopt_ptr(curl_sys::CURLOPT_WRITEDATA, ptr)
            .expect("failed to set write callback");

        let cb: curl_sys::curl_read_callback = read_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_READFUNCTION, cb as *const _)
            .expect("failed to set read callback");
        self.setopt_ptr(curl_sys::CURLOPT_READDATA, ptr)
            .expect("failed to set read callback");

        let cb: curl_sys::curl_seek_callback = seek_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_SEEKFUNCTION, cb as *const _)
            .expect("failed to set seek callback");
        self.setopt_ptr(curl_sys::CURLOPT_SEEKDATA, ptr)
            .expect("failed to set seek callback");

        let cb: curl_sys::curl_progress_callback = progress_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_PROGRESSFUNCTION, cb as *const _)
            .expect("failed to set progress callback");
        self.setopt_ptr(curl_sys::CURLOPT_PROGRESSDATA, ptr)
            .expect("failed to set progress callback");

        let cb: curl_sys::curl_debug_callback = debug_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_DEBUGFUNCTION, cb as *const _)
            .expect("failed to set debug callback");
        self.setopt_ptr(curl_sys::CURLOPT_DEBUGDATA, ptr)
            .expect("failed to set debug callback");

        let cb: curl_sys::curl_ssl_ctx_callback = ssl_ctx_cb::<H>;
        drop(self.setopt_ptr(curl_sys::CURLOPT_SSL_CTX_FUNCTION, cb as *const _));
        drop(self.setopt_ptr(curl_sys::CURLOPT_SSL_CTX_DATA, ptr));

        let cb: curl_sys::curl_opensocket_callback = opensocket_cb::<H>;
        self.setopt_ptr(curl_sys::CURLOPT_OPENSOCKETFUNCTION, cb as *const _)
            .expect("failed to set open socket callback");
        self.setopt_ptr(curl_sys::CURLOPT_OPENSOCKETDATA, ptr)
            .expect("failed to set open socket callback");
    }

    #[cfg(need_openssl_probe)]
    fn ssl_configure(&mut self) {
        let probe = ::openssl_probe::probe();
        if let Some(ref path) = probe.cert_file {
            let _ = self.cainfo(path);
        }
        if let Some(ref path) = probe.cert_dir {
            let _ = self.capath(path);
        }
    }

    #[cfg(not(need_openssl_probe))]
    fn ssl_configure(&mut self) {}
}

impl<H> Easy2<H> {
    // =========================================================================
    // Behavior options

    /// Configures this handle to have verbose output to help debug protocol
    /// information.
    ///
    /// By default output goes to stderr, but the `stderr` function on this type
    /// can configure that. You can also use the `debug_function` method to get
    /// all protocol data sent and received.
    ///
    /// By default, this option is `false`.
    pub fn verbose(&mut self, verbose: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_VERBOSE, verbose as c_long)
    }

    /// Indicates whether header information is streamed to the output body of
    /// this request.
    ///
    /// This option is only relevant for protocols which have header metadata
    /// (like http or ftp). It's not generally possible to extract headers
    /// from the body if using this method, that use case should be intended for
    /// the `header_function` method.
    ///
    /// To set HTTP headers, use the `http_header` method.
    ///
    /// By default, this option is `false` and corresponds to
    /// `CURLOPT_HEADER`.
    pub fn show_header(&mut self, show: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HEADER, show as c_long)
    }

    /// Indicates whether a progress meter will be shown for requests done with
    /// this handle.
    ///
    /// This will also prevent the `progress_function` from being called.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_NOPROGRESS`.
    pub fn progress(&mut self, progress: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_NOPROGRESS, (!progress) as c_long)
    }

    /// Inform libcurl whether or not it should install signal handlers or
    /// attempt to use signals to perform library functions.
    ///
    /// If this option is disabled then timeouts during name resolution will not
    /// work unless libcurl is built against c-ares. Note that enabling this
    /// option, however, may not cause libcurl to work with multiple threads.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_NOSIGNAL`.
    /// Note that this default is **different than libcurl** as it is intended
    /// that this library is threadsafe by default. See the [libcurl docs] for
    /// some more information.
    ///
    /// [libcurl docs]: https://curl.haxx.se/libcurl/c/threadsafe.html
    pub fn signal(&mut self, signal: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_NOSIGNAL, (!signal) as c_long)
    }

    /// Indicates whether multiple files will be transferred based on the file
    /// name pattern.
    ///
    /// The last part of a filename uses fnmatch-like pattern matching.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_WILDCARDMATCH`.
    pub fn wildcard_match(&mut self, m: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_WILDCARDMATCH, m as c_long)
    }

    /// Provides the unix domain socket which this handle will work with.
    ///
    /// The string provided must be unix domain socket -encoded with the format:
    ///
    /// ```text
    /// /path/file.sock
    /// ```
    pub fn unix_socket(&mut self, unix_domain_socket: &str) -> Result<(), Error> {
        let socket = CString::new(unix_domain_socket)?;
        self.setopt_str(curl_sys::CURLOPT_UNIX_SOCKET_PATH, &socket)
    }

    // =========================================================================
    // Internal accessors

    /// Acquires a reference to the underlying handler for events.
    pub fn get_ref(&self) -> &H {
        &self.inner.handler
    }

    /// Acquires a reference to the underlying handler for events.
    pub fn get_mut(&mut self) -> &mut H {
        &mut self.inner.handler
    }

    // =========================================================================
    // Error options

    // TODO: error buffer and stderr

    /// Indicates whether this library will fail on HTTP response codes >= 400.
    ///
    /// This method is not fail-safe especially when authentication is involved.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FAILONERROR`.
    pub fn fail_on_error(&mut self, fail: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_FAILONERROR, fail as c_long)
    }

    // =========================================================================
    // Network options

    /// Provides the URL which this handle will work with.
    ///
    /// The string provided must be URL-encoded with the format:
    ///
    /// ```text
    /// scheme://host:port/path
    /// ```
    ///
    /// The syntax is not validated as part of this function and that is
    /// deferred until later.
    ///
    /// By default this option is not set and `perform` will not work until it
    /// is set. This option corresponds to `CURLOPT_URL`.
    pub fn url(&mut self, url: &str) -> Result<(), Error> {
        let url = CString::new(url)?;
        self.setopt_str(curl_sys::CURLOPT_URL, &url)
    }

    /// Configures the port number to connect to, instead of the one specified
    /// in the URL or the default of the protocol.
    pub fn port(&mut self, port: u16) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_PORT, port as c_long)
    }

    // /// Indicates whether sequences of `/../` and `/./` will be squashed or not.
    // ///
    // /// By default this option is `false` and corresponds to
    // /// `CURLOPT_PATH_AS_IS`.
    // pub fn path_as_is(&mut self, as_is: bool) -> Result<(), Error> {
    // }

    /// Provide the URL of a proxy to use.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_PROXY`.
    pub fn proxy(&mut self, url: &str) -> Result<(), Error> {
        let url = CString::new(url)?;
        self.setopt_str(curl_sys::CURLOPT_PROXY, &url)
    }

    /// Provide port number the proxy is listening on.
    ///
    /// By default this option is not set (the default port for the proxy
    /// protocol is used) and corresponds to `CURLOPT_PROXYPORT`.
    pub fn proxy_port(&mut self, port: u16) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_PROXYPORT, port as c_long)
    }

    /// Set CA certificate to verify peer against for proxy
    ///
    /// By default this value is not set and corresponds to `CURLOPT_PROXY_CAINFO`.
    pub fn proxy_cainfo(&mut self, cainfo: &str) -> Result<(), Error> {
        let cainfo = CString::new(cainfo)?;
        self.setopt_str(curl_sys::CURLOPT_PROXY_CAINFO, &cainfo)
    }

    /// Set client certificate for proxy
    ///
    /// By default this value is not set and corresponds to `CURLOPT_PROXY_SSLCERT`.
    pub fn proxy_sslcert(&mut self, sslcert: &str) -> Result<(), Error> {
        let sslcert = CString::new(sslcert)?;
        self.setopt_str(curl_sys::CURLOPT_PROXY_SSLCERT, &sslcert)
    }

    /// Set private key for HTTPS proxy
    ///
    /// By default this value is not set and corresponds to `CURLOPT_PROXY_SSLKEY`.
    pub fn proxy_sslkey(&mut self, sslkey: &str) -> Result<(), Error> {
        let sslkey = CString::new(sslkey)?;
        self.setopt_str(curl_sys::CURLOPT_PROXY_SSLKEY, &sslkey)
    }

    /// Indicates the type of proxy being used.
    ///
    /// By default this option is `ProxyType::Http` and corresponds to
    /// `CURLOPT_PROXYTYPE`.
    pub fn proxy_type(&mut self, kind: ProxyType) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_PROXYTYPE, kind as c_long)
    }

    /// Provide a list of hosts that should not be proxied to.
    ///
    /// This string is a comma-separated list of hosts which should not use the
    /// proxy specified for connections. A single `*` character is also accepted
    /// as a wildcard for all hosts.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_NOPROXY`.
    pub fn noproxy(&mut self, skip: &str) -> Result<(), Error> {
        let skip = CString::new(skip)?;
        self.setopt_str(curl_sys::CURLOPT_NOPROXY, &skip)
    }

    /// Inform curl whether it should tunnel all operations through the proxy.
    ///
    /// This essentially means that a `CONNECT` is sent to the proxy for all
    /// outbound requests.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_HTTPPROXYTUNNEL`.
    pub fn http_proxy_tunnel(&mut self, tunnel: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HTTPPROXYTUNNEL, tunnel as c_long)
    }

    /// Tell curl which interface to bind to for an outgoing network interface.
    ///
    /// The interface name, IP address, or host name can be specified here.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INTERFACE`.
    pub fn interface(&mut self, interface: &str) -> Result<(), Error> {
        let s = CString::new(interface)?;
        self.setopt_str(curl_sys::CURLOPT_INTERFACE, &s)
    }

    /// Indicate which port should be bound to locally for this connection.
    ///
    /// By default this option is 0 (any port) and corresponds to
    /// `CURLOPT_LOCALPORT`.
    pub fn set_local_port(&mut self, port: u16) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_LOCALPORT, port as c_long)
    }

    /// Indicates the number of attempts libcurl will perform to find a working
    /// port number.
    ///
    /// By default this option is 1 and corresponds to
    /// `CURLOPT_LOCALPORTRANGE`.
    pub fn local_port_range(&mut self, range: u16) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_LOCALPORTRANGE, range as c_long)
    }

    /// Sets the DNS servers that wil be used.
    ///
    /// Provide a comma separated list, for example: `8.8.8.8,8.8.4.4`.
    ///
    /// By default this option is not set and the OS's DNS resolver is used.
    /// This option can only be used if libcurl is linked against
    /// [c-ares](https://c-ares.haxx.se), otherwise setting it will return
    /// an error.
    pub fn dns_servers(&mut self, servers: &str) -> Result<(), Error> {
        let s = CString::new(servers)?;
        self.setopt_str(curl_sys::CURLOPT_DNS_SERVERS, &s)
    }

    /// Sets the timeout of how long name resolves will be kept in memory.
    ///
    /// This is distinct from DNS TTL options and is entirely speculative.
    ///
    /// By default this option is 60s and corresponds to
    /// `CURLOPT_DNS_CACHE_TIMEOUT`.
    pub fn dns_cache_timeout(&mut self, dur: Duration) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_DNS_CACHE_TIMEOUT, dur.as_secs() as c_long)
    }

    /// Specify the preferred receive buffer size, in bytes.
    ///
    /// This is treated as a request, not an order, and the main point of this
    /// is that the write callback may get called more often with smaller
    /// chunks.
    ///
    /// By default this option is the maximum write size and corresopnds to
    /// `CURLOPT_BUFFERSIZE`.
    pub fn buffer_size(&mut self, size: usize) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_BUFFERSIZE, size as c_long)
    }

    // /// Enable or disable TCP Fast Open
    // ///
    // /// By default this options defaults to `false` and corresponds to
    // /// `CURLOPT_TCP_FASTOPEN`
    // pub fn fast_open(&mut self, enable: bool) -> Result<(), Error> {
    // }

    /// Configures whether the TCP_NODELAY option is set, or Nagle's algorithm
    /// is disabled.
    ///
    /// The purpose of Nagle's algorithm is to minimize the number of small
    /// packet's on the network, and disabling this may be less efficient in
    /// some situations.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_TCP_NODELAY`.
    pub fn tcp_nodelay(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TCP_NODELAY, enable as c_long)
    }

    /// Configures whether TCP keepalive probes will be sent.
    ///
    /// The delay and frequency of these probes is controlled by `tcp_keepidle`
    /// and `tcp_keepintvl`.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_TCP_KEEPALIVE`.
    pub fn tcp_keepalive(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TCP_KEEPALIVE, enable as c_long)
    }

    /// Configures the TCP keepalive idle time wait.
    ///
    /// This is the delay, after which the connection is idle, keepalive probes
    /// will be sent. Not all operating systems support this.
    ///
    /// By default this corresponds to `CURLOPT_TCP_KEEPIDLE`.
    pub fn tcp_keepidle(&mut self, amt: Duration) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TCP_KEEPIDLE, amt.as_secs() as c_long)
    }

    /// Configures the delay between keepalive probes.
    ///
    /// By default this corresponds to `CURLOPT_TCP_KEEPINTVL`.
    pub fn tcp_keepintvl(&mut self, amt: Duration) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TCP_KEEPINTVL, amt.as_secs() as c_long)
    }

    /// Configures the scope for local IPv6 addresses.
    ///
    /// Sets the scope_id value to use when connecting to IPv6 or link-local
    /// addresses.
    ///
    /// By default this value is 0 and corresponds to `CURLOPT_ADDRESS_SCOPE`
    pub fn address_scope(&mut self, scope: u32) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_ADDRESS_SCOPE, scope as c_long)
    }

    // =========================================================================
    // Names and passwords

    /// Configures the username to pass as authentication for this connection.
    ///
    /// By default this value is not set and corresponds to `CURLOPT_USERNAME`.
    pub fn username(&mut self, user: &str) -> Result<(), Error> {
        let user = CString::new(user)?;
        self.setopt_str(curl_sys::CURLOPT_USERNAME, &user)
    }

    /// Configures the password to pass as authentication for this connection.
    ///
    /// By default this value is not set and corresponds to `CURLOPT_PASSWORD`.
    pub fn password(&mut self, pass: &str) -> Result<(), Error> {
        let pass = CString::new(pass)?;
        self.setopt_str(curl_sys::CURLOPT_PASSWORD, &pass)
    }

    /// Set HTTP server authentication methods to try
    ///
    /// If more than one method is set, libcurl will first query the site to see
    /// which authentication methods it supports and then pick the best one you
    /// allow it to use. For some methods, this will induce an extra network
    /// round-trip. Set the actual name and password with the `password` and
    /// `username` methods.
    ///
    /// For authentication with a proxy, see `proxy_auth`.
    ///
    /// By default this value is basic and corresponds to `CURLOPT_HTTPAUTH`.
    pub fn http_auth(&mut self, auth: &Auth) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HTTPAUTH, auth.bits)
    }

    /// Configures the proxy username to pass as authentication for this
    /// connection.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXYUSERNAME`.
    pub fn proxy_username(&mut self, user: &str) -> Result<(), Error> {
        let user = CString::new(user)?;
        self.setopt_str(curl_sys::CURLOPT_PROXYUSERNAME, &user)
    }

    /// Configures the proxy password to pass as authentication for this
    /// connection.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXYPASSWORD`.
    pub fn proxy_password(&mut self, pass: &str) -> Result<(), Error> {
        let pass = CString::new(pass)?;
        self.setopt_str(curl_sys::CURLOPT_PROXYPASSWORD, &pass)
    }

    /// Set HTTP proxy authentication methods to try
    ///
    /// If more than one method is set, libcurl will first query the site to see
    /// which authentication methods it supports and then pick the best one you
    /// allow it to use. For some methods, this will induce an extra network
    /// round-trip. Set the actual name and password with the `proxy_password`
    /// and `proxy_username` methods.
    ///
    /// By default this value is basic and corresponds to `CURLOPT_PROXYAUTH`.
    pub fn proxy_auth(&mut self, auth: &Auth) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_PROXYAUTH, auth.bits)
    }

    /// Enable .netrc parsing
    ///
    /// By default the .netrc file is ignored and corresponds to `CURL_NETRC_IGNORED`.
    pub fn netrc(&mut self, netrc: NetRc) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_NETRC, netrc as c_long)
    }

    // =========================================================================
    // HTTP Options

    /// Indicates whether the referer header is automatically updated
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_AUTOREFERER`.
    pub fn autoreferer(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_AUTOREFERER, enable as c_long)
    }

    /// Enables automatic decompression of HTTP downloads.
    ///
    /// Sets the contents of the Accept-Encoding header sent in an HTTP request.
    /// This enables decoding of a response with Content-Encoding.
    ///
    /// Currently supported encoding are `identity`, `zlib`, and `gzip`. A
    /// zero-length string passed in will send all accepted encodings.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_ACCEPT_ENCODING`.
    pub fn accept_encoding(&mut self, encoding: &str) -> Result<(), Error> {
        let encoding = CString::new(encoding)?;
        self.setopt_str(curl_sys::CURLOPT_ACCEPT_ENCODING, &encoding)
    }

    /// Request the HTTP Transfer Encoding.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_TRANSFER_ENCODING`.
    pub fn transfer_encoding(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TRANSFER_ENCODING, enable as c_long)
    }

    /// Follow HTTP 3xx redirects.
    ///
    /// Indicates whether any `Location` headers in the response should get
    /// followed.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FOLLOWLOCATION`.
    pub fn follow_location(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_FOLLOWLOCATION, enable as c_long)
    }

    /// Send credentials to hosts other than the first as well.
    ///
    /// Sends username/password credentials even when the host changes as part
    /// of a redirect.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_UNRESTRICTED_AUTH`.
    pub fn unrestricted_auth(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_UNRESTRICTED_AUTH, enable as c_long)
    }

    /// Set the maximum number of redirects allowed.
    ///
    /// A value of 0 will refuse any redirect.
    ///
    /// By default this option is `-1` (unlimited) and corresponds to
    /// `CURLOPT_MAXREDIRS`.
    pub fn max_redirections(&mut self, max: u32) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_MAXREDIRS, max as c_long)
    }

    // TODO: post_redirections

    /// Make an HTTP PUT request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_PUT`.
    pub fn put(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_PUT, enable as c_long)
    }

    /// Make an HTTP POST request.
    ///
    /// This will also make the library use the
    /// `Content-Type: application/x-www-form-urlencoded` header.
    ///
    /// POST data can be specified through `post_fields` or by specifying a read
    /// function.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_POST`.
    pub fn post(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_POST, enable as c_long)
    }

    /// Configures the data that will be uploaded as part of a POST.
    ///
    /// Note that the data is copied into this handle and if that's not desired
    /// then the read callbacks can be used instead.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_COPYPOSTFIELDS`.
    pub fn post_fields_copy(&mut self, data: &[u8]) -> Result<(), Error> {
        // Set the length before the pointer so libcurl knows how much to read
        self.post_field_size(data.len() as u64)?;
        self.setopt_ptr(curl_sys::CURLOPT_COPYPOSTFIELDS, data.as_ptr() as *const _)
    }

    /// Configures the size of data that's going to be uploaded as part of a
    /// POST operation.
    ///
    /// This is called automatically as part of `post_fields` and should only
    /// be called if data is being provided in a read callback (and even then
    /// it's optional).
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_POSTFIELDSIZE_LARGE`.
    pub fn post_field_size(&mut self, size: u64) -> Result<(), Error> {
        // Clear anything previous to ensure we don't read past a buffer
        self.setopt_ptr(curl_sys::CURLOPT_POSTFIELDS, 0 as *const _)?;
        self.setopt_off_t(
            curl_sys::CURLOPT_POSTFIELDSIZE_LARGE,
            size as curl_sys::curl_off_t,
        )
    }

    /// Tells libcurl you want a multipart/formdata HTTP POST to be made and you
    /// instruct what data to pass on to the server in the `form` argument.
    ///
    /// By default this option is set to null and corresponds to
    /// `CURLOPT_HTTPPOST`.
    pub fn httppost(&mut self, form: Form) -> Result<(), Error> {
        self.setopt_ptr(curl_sys::CURLOPT_HTTPPOST, form::raw(&form) as *const _)?;
        self.inner.form = Some(form);
        Ok(())
    }

    /// Sets the HTTP referer header
    ///
    /// By default this option is not set and corresponds to `CURLOPT_REFERER`.
    pub fn referer(&mut self, referer: &str) -> Result<(), Error> {
        let referer = CString::new(referer)?;
        self.setopt_str(curl_sys::CURLOPT_REFERER, &referer)
    }

    /// Sets the HTTP user-agent header
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_USERAGENT`.
    pub fn useragent(&mut self, useragent: &str) -> Result<(), Error> {
        let useragent = CString::new(useragent)?;
        self.setopt_str(curl_sys::CURLOPT_USERAGENT, &useragent)
    }

    /// Add some headers to this HTTP request.
    ///
    /// If you add a header that is otherwise used internally, the value here
    /// takes precedence. If a header is added with no content (like `Accept:`)
    /// the internally the header will get disabled. To add a header with no
    /// content, use the form `MyHeader;` (not the trailing semicolon).
    ///
    /// Headers must not be CRLF terminated. Many replaced headers have common
    /// shortcuts which should be prefered.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_HTTPHEADER`
    ///
    /// # Examples
    ///
    /// ```
    /// use curl::easy::{Easy, List};
    ///
    /// let mut list = List::new();
    /// list.append("Foo: bar").unwrap();
    /// list.append("Bar: baz").unwrap();
    ///
    /// let mut handle = Easy::new();
    /// handle.url("https://www.rust-lang.org/").unwrap();
    /// handle.http_headers(list).unwrap();
    /// handle.perform().unwrap();
    /// ```
    pub fn http_headers(&mut self, list: List) -> Result<(), Error> {
        let ptr = list::raw(&list);
        self.inner.header_list = Some(list);
        self.setopt_ptr(curl_sys::CURLOPT_HTTPHEADER, ptr as *const _)
    }

    // /// Add some headers to send to the HTTP proxy.
    // ///
    // /// This function is essentially the same as `http_headers`.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_PROXYHEADER`
    // pub fn proxy_headers(&mut self, list: &'a List) -> Result<(), Error> {
    //     self.setopt_ptr(curl_sys::CURLOPT_PROXYHEADER, list.raw as *const _)
    // }

    /// Set the contents of the HTTP Cookie header.
    ///
    /// Pass a string of the form `name=contents` for one cookie value or
    /// `name1=val1; name2=val2` for multiple values.
    ///
    /// Using this option multiple times will only make the latest string
    /// override the previous ones. This option will not enable the cookie
    /// engine, use `cookie_file` or `cookie_jar` to do that.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_COOKIE`.
    pub fn cookie(&mut self, cookie: &str) -> Result<(), Error> {
        let cookie = CString::new(cookie)?;
        self.setopt_str(curl_sys::CURLOPT_COOKIE, &cookie)
    }

    /// Set the file name to read cookies from.
    ///
    /// The cookie data can be in either the old Netscape / Mozilla cookie data
    /// format or just regular HTTP headers (Set-Cookie style) dumped to a file.
    ///
    /// This also enables the cookie engine, making libcurl parse and send
    /// cookies on subsequent requests with this handle.
    ///
    /// Given an empty or non-existing file or by passing the empty string ("")
    /// to this option, you can enable the cookie engine without reading any
    /// initial cookies.
    ///
    /// If you use this option multiple times, you just add more files to read.
    /// Subsequent files will add more cookies.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_COOKIEFILE`.
    pub fn cookie_file<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_COOKIEFILE, file.as_ref())
    }

    /// Set the file name to store cookies to.
    ///
    /// This will make libcurl write all internally known cookies to the file
    /// when this handle is dropped. If no cookies are known, no file will be
    /// created. Specify "-" as filename to instead have the cookies written to
    /// stdout. Using this option also enables cookies for this session, so if
    /// you for example follow a location it will make matching cookies get sent
    /// accordingly.
    ///
    /// Note that libcurl doesn't read any cookies from the cookie jar. If you
    /// want to read cookies from a file, use `cookie_file`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_COOKIEJAR`.
    pub fn cookie_jar<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_COOKIEJAR, file.as_ref())
    }

    /// Start a new cookie session
    ///
    /// Marks this as a new cookie "session". It will force libcurl to ignore
    /// all cookies it is about to load that are "session cookies" from the
    /// previous session. By default, libcurl always stores and loads all
    /// cookies, independent if they are session cookies or not. Session cookies
    /// are cookies without expiry date and they are meant to be alive and
    /// existing for this "session" only.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_COOKIESESSION`.
    pub fn cookie_session(&mut self, session: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_COOKIESESSION, session as c_long)
    }

    /// Add to or manipulate cookies held in memory.
    ///
    /// Such a cookie can be either a single line in Netscape / Mozilla format
    /// or just regular HTTP-style header (Set-Cookie: ...) format. This will
    /// also enable the cookie engine. This adds that single cookie to the
    /// internal cookie store.
    ///
    /// Exercise caution if you are using this option and multiple transfers may
    /// occur. If you use the Set-Cookie format and don't specify a domain then
    /// the cookie is sent for any domain (even after redirects are followed)
    /// and cannot be modified by a server-set cookie. If a server sets a cookie
    /// of the same name (or maybe you've imported one) then both will be sent
    /// on a future transfer to that server, likely not what you intended.
    /// address these issues set a domain in Set-Cookie or use the Netscape
    /// format.
    ///
    /// Additionally, there are commands available that perform actions if you
    /// pass in these exact strings:
    ///
    /// * "ALL" - erases all cookies held in memory
    /// * "SESS" - erases all session cookies held in memory
    /// * "FLUSH" - write all known cookies to the specified cookie jar
    /// * "RELOAD" - reread all cookies from the cookie file
    ///
    /// By default this options corresponds to `CURLOPT_COOKIELIST`
    pub fn cookie_list(&mut self, cookie: &str) -> Result<(), Error> {
        let cookie = CString::new(cookie)?;
        self.setopt_str(curl_sys::CURLOPT_COOKIELIST, &cookie)
    }

    /// Ask for a HTTP GET request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_HTTPGET`.
    pub fn get(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HTTPGET, enable as c_long)
    }

    // /// Ask for a HTTP GET request.
    // ///
    // /// By default this option is `false` and corresponds to `CURLOPT_HTTPGET`.
    // pub fn http_version(&mut self, vers: &str) -> Result<(), Error> {
    //     self.setopt_long(curl_sys::CURLOPT_HTTPGET, enable as c_long)
    // }

    /// Ignore the content-length header.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_IGNORE_CONTENT_LENGTH`.
    pub fn ignore_content_length(&mut self, ignore: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_IGNORE_CONTENT_LENGTH, ignore as c_long)
    }

    /// Enable or disable HTTP content decoding.
    ///
    /// By default this option is `true` and corresponds to
    /// `CURLOPT_HTTP_CONTENT_DECODING`.
    pub fn http_content_decoding(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HTTP_CONTENT_DECODING, enable as c_long)
    }

    /// Enable or disable HTTP transfer decoding.
    ///
    /// By default this option is `true` and corresponds to
    /// `CURLOPT_HTTP_TRANSFER_DECODING`.
    pub fn http_transfer_decoding(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HTTP_TRANSFER_DECODING, enable as c_long)
    }

    // /// Timeout for the Expect: 100-continue response
    // ///
    // /// By default this option is 1s and corresponds to
    // /// `CURLOPT_EXPECT_100_TIMEOUT_MS`.
    // pub fn expect_100_timeout(&mut self, enable: bool) -> Result<(), Error> {
    //     self.setopt_long(curl_sys::CURLOPT_HTTP_TRANSFER_DECODING,
    //                      enable as c_long)
    // }

    // /// Wait for pipelining/multiplexing.
    // ///
    // /// Tells libcurl to prefer to wait for a connection to confirm or deny that
    // /// it can do pipelining or multiplexing before continuing.
    // ///
    // /// When about to perform a new transfer that allows pipelining or
    // /// multiplexing, libcurl will check for existing connections to re-use and
    // /// pipeline on. If no such connection exists it will immediately continue
    // /// and create a fresh new connection to use.
    // ///
    // /// By setting this option to `true` - having `pipeline` enabled for the
    // /// multi handle this transfer is associated with - libcurl will instead
    // /// wait for the connection to reveal if it is possible to
    // /// pipeline/multiplex on before it continues. This enables libcurl to much
    // /// better keep the number of connections to a minimum when using pipelining
    // /// or multiplexing protocols.
    // ///
    // /// The effect thus becomes that with this option set, libcurl prefers to
    // /// wait and re-use an existing connection for pipelining rather than the
    // /// opposite: prefer to open a new connection rather than waiting.
    // ///
    // /// The waiting time is as long as it takes for the connection to get up and
    // /// for libcurl to get the necessary response back that informs it about its
    // /// protocol and support level.
    // pub fn http_pipewait(&mut self, enable: bool) -> Result<(), Error> {
    // }

    // =========================================================================
    // Protocol Options

    /// Indicates the range that this request should retrieve.
    ///
    /// The string provided should be of the form `N-M` where either `N` or `M`
    /// can be left out. For HTTP transfers multiple ranges separated by commas
    /// are also accepted.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_RANGE`.
    pub fn range(&mut self, range: &str) -> Result<(), Error> {
        let range = CString::new(range)?;
        self.setopt_str(curl_sys::CURLOPT_RANGE, &range)
    }

    /// Set a point to resume transfer from
    ///
    /// Specify the offset in bytes you want the transfer to start from.
    ///
    /// By default this option is 0 and corresponds to
    /// `CURLOPT_RESUME_FROM_LARGE`.
    pub fn resume_from(&mut self, from: u64) -> Result<(), Error> {
        self.setopt_off_t(
            curl_sys::CURLOPT_RESUME_FROM_LARGE,
            from as curl_sys::curl_off_t,
        )
    }

    /// Set a custom request string
    ///
    /// Specifies that a custom request will be made (e.g. a custom HTTP
    /// method). This does not change how libcurl performs internally, just
    /// changes the string sent to the server.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_CUSTOMREQUEST`.
    pub fn custom_request(&mut self, request: &str) -> Result<(), Error> {
        let request = CString::new(request)?;
        self.setopt_str(curl_sys::CURLOPT_CUSTOMREQUEST, &request)
    }

    /// Get the modification time of the remote resource
    ///
    /// If true, libcurl will attempt to get the modification time of the
    /// remote document in this operation. This requires that the remote server
    /// sends the time or replies to a time querying command. The `filetime`
    /// function can be used after a transfer to extract the received time (if
    /// any).
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_FILETIME`
    pub fn fetch_filetime(&mut self, fetch: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_FILETIME, fetch as c_long)
    }

    /// Indicate whether to download the request without getting the body
    ///
    /// This is useful, for example, for doing a HEAD request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_NOBODY`.
    pub fn nobody(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_NOBODY, enable as c_long)
    }

    /// Set the size of the input file to send off.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INFILESIZE_LARGE`.
    pub fn in_filesize(&mut self, size: u64) -> Result<(), Error> {
        self.setopt_off_t(
            curl_sys::CURLOPT_INFILESIZE_LARGE,
            size as curl_sys::curl_off_t,
        )
    }

    /// Enable or disable data upload.
    ///
    /// This means that a PUT request will be made for HTTP and probably wants
    /// to be combined with the read callback as well as the `in_filesize`
    /// method.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_UPLOAD`.
    pub fn upload(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_UPLOAD, enable as c_long)
    }

    /// Configure the maximum file size to download.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_MAXFILESIZE_LARGE`.
    pub fn max_filesize(&mut self, size: u64) -> Result<(), Error> {
        self.setopt_off_t(
            curl_sys::CURLOPT_MAXFILESIZE_LARGE,
            size as curl_sys::curl_off_t,
        )
    }

    /// Selects a condition for a time request.
    ///
    /// This value indicates how the `time_value` option is interpreted.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMECONDITION`.
    pub fn time_condition(&mut self, cond: TimeCondition) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TIMECONDITION, cond as c_long)
    }

    /// Sets the time value for a conditional request.
    ///
    /// The value here should be the number of seconds elapsed since January 1,
    /// 1970. To pass how to interpret this value, use `time_condition`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMEVALUE`.
    pub fn time_value(&mut self, val: i64) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_TIMEVALUE, val as c_long)
    }

    // =========================================================================
    // Connection Options

    /// Set maximum time the request is allowed to take.
    ///
    /// Normally, name lookups can take a considerable time and limiting
    /// operations to less than a few minutes risk aborting perfectly normal
    /// operations.
    ///
    /// If libcurl is built to use the standard system name resolver, that
    /// portion of the transfer will still use full-second resolution for
    /// timeouts with a minimum timeout allowed of one second.
    ///
    /// In unix-like systems, this might cause signals to be used unless
    /// `nosignal` is set.
    ///
    /// Since this puts a hard limit for how long a request is allowed to
    /// take, it has limited use in dynamic use cases with varying transfer
    /// times. You are then advised to explore `low_speed_limit`,
    /// `low_speed_time` or using `progress_function` to implement your own
    /// timeout logic.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMEOUT_MS`.
    pub fn timeout(&mut self, timeout: Duration) -> Result<(), Error> {
        // TODO: checked arithmetic and casts
        // TODO: use CURLOPT_TIMEOUT if the timeout is too great
        let ms = timeout.as_secs() * 1000 + (timeout.subsec_nanos() / 1_000_000) as u64;
        self.setopt_long(curl_sys::CURLOPT_TIMEOUT_MS, ms as c_long)
    }

    /// Set the low speed limit in bytes per second.
    ///
    /// This specifies the average transfer speed in bytes per second that the
    /// transfer should be below during `low_speed_time` for libcurl to consider
    /// it to be too slow and abort.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_LOW_SPEED_LIMIT`.
    pub fn low_speed_limit(&mut self, limit: u32) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_LOW_SPEED_LIMIT, limit as c_long)
    }

    /// Set the low speed time period.
    ///
    /// Specifies the window of time for which if the transfer rate is below
    /// `low_speed_limit` the request will be aborted.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_LOW_SPEED_TIME`.
    pub fn low_speed_time(&mut self, dur: Duration) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_LOW_SPEED_TIME, dur.as_secs() as c_long)
    }

    /// Rate limit data upload speed
    ///
    /// If an upload exceeds this speed (counted in bytes per second) on
    /// cumulative average during the transfer, the transfer will pause to keep
    /// the average rate less than or equal to the parameter value.
    ///
    /// By default this option is not set (unlimited speed) and corresponds to
    /// `CURLOPT_MAX_SEND_SPEED_LARGE`.
    pub fn max_send_speed(&mut self, speed: u64) -> Result<(), Error> {
        self.setopt_off_t(
            curl_sys::CURLOPT_MAX_SEND_SPEED_LARGE,
            speed as curl_sys::curl_off_t,
        )
    }

    /// Rate limit data download speed
    ///
    /// If a download exceeds this speed (counted in bytes per second) on
    /// cumulative average during the transfer, the transfer will pause to keep
    /// the average rate less than or equal to the parameter value.
    ///
    /// By default this option is not set (unlimited speed) and corresponds to
    /// `CURLOPT_MAX_RECV_SPEED_LARGE`.
    pub fn max_recv_speed(&mut self, speed: u64) -> Result<(), Error> {
        self.setopt_off_t(
            curl_sys::CURLOPT_MAX_RECV_SPEED_LARGE,
            speed as curl_sys::curl_off_t,
        )
    }

    /// Set the maximum connection cache size.
    ///
    /// The set amount will be the maximum number of simultaneously open
    /// persistent connections that libcurl may cache in the pool associated
    /// with this handle. The default is 5, and there isn't much point in
    /// changing this value unless you are perfectly aware of how this works and
    /// changes libcurl's behaviour. This concerns connections using any of the
    /// protocols that support persistent connections.
    ///
    /// When reaching the maximum limit, curl closes the oldest one in the cache
    /// to prevent increasing the number of open connections.
    ///
    /// By default this option is set to 5 and corresponds to
    /// `CURLOPT_MAXCONNECTS`
    pub fn max_connects(&mut self, max: u32) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_MAXCONNECTS, max as c_long)
    }

    /// Force a new connection to be used.
    ///
    /// Makes the next transfer use a new (fresh) connection by force instead of
    /// trying to re-use an existing one. This option should be used with
    /// caution and only if you understand what it does as it may seriously
    /// impact performance.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FRESH_CONNECT`.
    pub fn fresh_connect(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_FRESH_CONNECT, enable as c_long)
    }

    /// Make connection get closed at once after use.
    ///
    /// Makes libcurl explicitly close the connection when done with the
    /// transfer. Normally, libcurl keeps all connections alive when done with
    /// one transfer in case a succeeding one follows that can re-use them.
    /// This option should be used with caution and only if you understand what
    /// it does as it can seriously impact performance.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FORBID_REUSE`.
    pub fn forbid_reuse(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_FORBID_REUSE, enable as c_long)
    }

    /// Timeout for the connect phase
    ///
    /// This is the maximum time that you allow the connection phase to the
    /// server to take. This only limits the connection phase, it has no impact
    /// once it has connected.
    ///
    /// By default this value is 300 seconds and corresponds to
    /// `CURLOPT_CONNECTTIMEOUT_MS`.
    pub fn connect_timeout(&mut self, timeout: Duration) -> Result<(), Error> {
        let ms = timeout.as_secs() * 1000 + (timeout.subsec_nanos() / 1_000_000) as u64;
        self.setopt_long(curl_sys::CURLOPT_CONNECTTIMEOUT_MS, ms as c_long)
    }

    /// Specify which IP protocol version to use
    ///
    /// Allows an application to select what kind of IP addresses to use when
    /// resolving host names. This is only interesting when using host names
    /// that resolve addresses using more than one version of IP.
    ///
    /// By default this value is "any" and corresponds to `CURLOPT_IPRESOLVE`.
    pub fn ip_resolve(&mut self, resolve: IpResolve) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_IPRESOLVE, resolve as c_long)
    }

    /// Specify custom host name to IP address resolves.
    ///
    /// Allows specifying hostname to IP mappins to use before trying the
    /// system resolver.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use curl::easy::{Easy, List};
    ///
    /// let mut list = List::new();
    /// list.append("www.rust-lang.org:443:185.199.108.153").unwrap();
    ///
    /// let mut handle = Easy::new();
    /// handle.url("https://www.rust-lang.org/").unwrap();
    /// handle.resolve(list).unwrap();
    /// handle.perform().unwrap();
    /// ```
    pub fn resolve(&mut self, list: List) -> Result<(), Error> {
        let ptr = list::raw(&list);
        self.inner.resolve_list = Some(list);
        self.setopt_ptr(curl_sys::CURLOPT_RESOLVE, ptr as *const _)
    }

    /// Configure whether to stop when connected to target server
    ///
    /// When enabled it tells the library to perform all the required proxy
    /// authentication and connection setup, but no data transfer, and then
    /// return.
    ///
    /// The option can be used to simply test a connection to a server.
    ///
    /// By default this value is `false` and corresponds to
    /// `CURLOPT_CONNECT_ONLY`.
    pub fn connect_only(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_CONNECT_ONLY, enable as c_long)
    }

    // /// Set interface to speak DNS over.
    // ///
    // /// Set the name of the network interface that the DNS resolver should bind
    // /// to. This must be an interface name (not an address).
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_DNS_INTERFACE`.
    // pub fn dns_interface(&mut self, interface: &str) -> Result<(), Error> {
    //     let interface = CString::new(interface)?;
    //     self.setopt_str(curl_sys::CURLOPT_DNS_INTERFACE, &interface)
    // }
    //
    // /// IPv4 address to bind DNS resolves to
    // ///
    // /// Set the local IPv4 address that the resolver should bind to. The
    // /// argument should be of type char * and contain a single numerical IPv4
    // /// address as a string.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_DNS_LOCAL_IP4`.
    // pub fn dns_local_ip4(&mut self, ip: &str) -> Result<(), Error> {
    //     let ip = CString::new(ip)?;
    //     self.setopt_str(curl_sys::CURLOPT_DNS_LOCAL_IP4, &ip)
    // }
    //
    // /// IPv6 address to bind DNS resolves to
    // ///
    // /// Set the local IPv6 address that the resolver should bind to. The
    // /// argument should be of type char * and contain a single numerical IPv6
    // /// address as a string.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_DNS_LOCAL_IP6`.
    // pub fn dns_local_ip6(&mut self, ip: &str) -> Result<(), Error> {
    //     let ip = CString::new(ip)?;
    //     self.setopt_str(curl_sys::CURLOPT_DNS_LOCAL_IP6, &ip)
    // }
    //
    // /// Set preferred DNS servers.
    // ///
    // /// Provides a list of DNS servers to be used instead of the system default.
    // /// The format of the dns servers option is:
    // ///
    // /// ```text
    // /// host[:port],[host[:port]]...
    // /// ```
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_DNS_SERVERS`.
    // pub fn dns_servers(&mut self, servers: &str) -> Result<(), Error> {
    //     let servers = CString::new(servers)?;
    //     self.setopt_str(curl_sys::CURLOPT_DNS_SERVERS, &servers)
    // }

    // =========================================================================
    // SSL/Security Options

    /// Sets the SSL client certificate.
    ///
    /// The string should be the file name of your client certificate. The
    /// default format is "P12" on Secure Transport and "PEM" on other engines,
    /// and can be changed with `ssl_cert_type`.
    ///
    /// With NSS or Secure Transport, this can also be the nickname of the
    /// certificate you wish to authenticate with as it is named in the security
    /// database. If you want to use a file from the current directory, please
    /// precede it with "./" prefix, in order to avoid confusion with a
    /// nickname.
    ///
    /// When using a client certificate, you most likely also need to provide a
    /// private key with `ssl_key`.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_SSLCERT`.
    pub fn ssl_cert<P: AsRef<Path>>(&mut self, cert: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_SSLCERT, cert.as_ref())
    }

    /// Specify type of the client SSL certificate.
    ///
    /// The string should be the format of your certificate. Supported formats
    /// are "PEM" and "DER", except with Secure Transport. OpenSSL (versions
    /// 0.9.3 and later) and Secure Transport (on iOS 5 or later, or OS X 10.7
    /// or later) also support "P12" for PKCS#12-encoded files.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_SSLCERTTYPE`.
    pub fn ssl_cert_type(&mut self, kind: &str) -> Result<(), Error> {
        let kind = CString::new(kind)?;
        self.setopt_str(curl_sys::CURLOPT_SSLCERTTYPE, &kind)
    }

    /// Specify private keyfile for TLS and SSL client cert.
    ///
    /// The string should be the file name of your private key. The default
    /// format is "PEM" and can be changed with `ssl_key_type`.
    ///
    /// (iOS and Mac OS X only) This option is ignored if curl was built against
    /// Secure Transport. Secure Transport expects the private key to be already
    /// present in the keychain or PKCS#12 file containing the certificate.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_SSLKEY`.
    pub fn ssl_key<P: AsRef<Path>>(&mut self, key: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_SSLKEY, key.as_ref())
    }

    /// Set type of the private key file.
    ///
    /// The string should be the format of your private key. Supported formats
    /// are "PEM", "DER" and "ENG".
    ///
    /// The format "ENG" enables you to load the private key from a crypto
    /// engine. In this case `ssl_key` is used as an identifier passed to
    /// the engine. You have to set the crypto engine with `ssl_engine`.
    /// "DER" format key file currently does not work because of a bug in
    /// OpenSSL.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_SSLKEYTYPE`.
    pub fn ssl_key_type(&mut self, kind: &str) -> Result<(), Error> {
        let kind = CString::new(kind)?;
        self.setopt_str(curl_sys::CURLOPT_SSLKEYTYPE, &kind)
    }

    /// Set passphrase to private key.
    ///
    /// This will be used as the password required to use the `ssl_key`.
    /// You never needed a pass phrase to load a certificate but you need one to
    /// load your private key.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_KEYPASSWD`.
    pub fn key_password(&mut self, password: &str) -> Result<(), Error> {
        let password = CString::new(password)?;
        self.setopt_str(curl_sys::CURLOPT_KEYPASSWD, &password)
    }

    /// Set the SSL engine identifier.
    ///
    /// This will be used as the identifier for the crypto engine you want to
    /// use for your private key.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLENGINE`.
    pub fn ssl_engine(&mut self, engine: &str) -> Result<(), Error> {
        let engine = CString::new(engine)?;
        self.setopt_str(curl_sys::CURLOPT_SSLENGINE, &engine)
    }

    /// Make this handle's SSL engine the default.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLENGINE_DEFAULT`.
    pub fn ssl_engine_default(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_SSLENGINE_DEFAULT, enable as c_long)
    }

    // /// Enable TLS false start.
    // ///
    // /// This option determines whether libcurl should use false start during the
    // /// TLS handshake. False start is a mode where a TLS client will start
    // /// sending application data before verifying the server's Finished message,
    // /// thus saving a round trip when performing a full handshake.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_SSL_FALSESTARTE`.
    // pub fn ssl_false_start(&mut self, enable: bool) -> Result<(), Error> {
    //     self.setopt_long(curl_sys::CURLOPT_SSLENGINE_DEFAULT, enable as c_long)
    // }

    /// Set preferred HTTP version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_HTTP_VERSION`.
    pub fn http_version(&mut self, version: HttpVersion) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_HTTP_VERSION, version as c_long)
    }

    /// Set preferred TLS/SSL version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLVERSION`.
    pub fn ssl_version(&mut self, version: SslVersion) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_SSLVERSION, version as c_long)
    }

    /// Set preferred TLS/SSL version with minimum version and maximum version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLVERSION`.
    pub fn ssl_min_max_version(
        &mut self,
        min_version: SslVersion,
        max_version: SslVersion,
    ) -> Result<(), Error> {
        let version = (min_version as c_long) | ((max_version as c_long) << 16);
        self.setopt_long(curl_sys::CURLOPT_SSLVERSION, version)
    }

    /// Verify the certificate's name against host.
    ///
    /// This should be disabled with great caution! It basically disables the
    /// security features of SSL if it is disabled.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_SSL_VERIFYHOST`.
    pub fn ssl_verify_host(&mut self, verify: bool) -> Result<(), Error> {
        let val = if verify { 2 } else { 0 };
        self.setopt_long(curl_sys::CURLOPT_SSL_VERIFYHOST, val)
    }

    /// Verify the peer's SSL certificate.
    ///
    /// This should be disabled with great caution! It basically disables the
    /// security features of SSL if it is disabled.
    ///
    /// By default this option is set to `true` and corresponds to
    /// `CURLOPT_SSL_VERIFYPEER`.
    pub fn ssl_verify_peer(&mut self, verify: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_SSL_VERIFYPEER, verify as c_long)
    }

    // /// Verify the certificate's status.
    // ///
    // /// This option determines whether libcurl verifies the status of the server
    // /// cert using the "Certificate Status Request" TLS extension (aka. OCSP
    // /// stapling).
    // ///
    // /// By default this option is set to `false` and corresponds to
    // /// `CURLOPT_SSL_VERIFYSTATUS`.
    // pub fn ssl_verify_status(&mut self, verify: bool) -> Result<(), Error> {
    //     self.setopt_long(curl_sys::CURLOPT_SSL_VERIFYSTATUS, verify as c_long)
    // }

    /// Specify the path to Certificate Authority (CA) bundle
    ///
    /// The file referenced should hold one or more certificates to verify the
    /// peer with.
    ///
    /// This option is by default set to the system path where libcurl's cacert
    /// bundle is assumed to be stored, as established at build time.
    ///
    /// If curl is built against the NSS SSL library, the NSS PEM PKCS#11 module
    /// (libnsspem.so) needs to be available for this option to work properly.
    ///
    /// By default this option is the system defaults, and corresponds to
    /// `CURLOPT_CAINFO`.
    pub fn cainfo<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_CAINFO, path.as_ref())
    }

    /// Set the issuer SSL certificate filename
    ///
    /// Specifies a file holding a CA certificate in PEM format. If the option
    /// is set, an additional check against the peer certificate is performed to
    /// verify the issuer is indeed the one associated with the certificate
    /// provided by the option. This additional check is useful in multi-level
    /// PKI where one needs to enforce that the peer certificate is from a
    /// specific branch of the tree.
    ///
    /// This option makes sense only when used in combination with the
    /// `ssl_verify_peer` option. Otherwise, the result of the check is not
    /// considered as failure.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_ISSUERCERT`.
    pub fn issuer_cert<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_ISSUERCERT, path.as_ref())
    }

    /// Specify directory holding CA certificates
    ///
    /// Names a directory holding multiple CA certificates to verify the peer
    /// with. If libcurl is built against OpenSSL, the certificate directory
    /// must be prepared using the openssl c_rehash utility. This makes sense
    /// only when used in combination with the `ssl_verify_peer` option.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_CAPATH`.
    pub fn capath<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_CAPATH, path.as_ref())
    }

    /// Specify a Certificate Revocation List file
    ///
    /// Names a file with the concatenation of CRL (in PEM format) to use in the
    /// certificate validation that occurs during the SSL exchange.
    ///
    /// When curl is built to use NSS or GnuTLS, there is no way to influence
    /// the use of CRL passed to help in the verification process. When libcurl
    /// is built with OpenSSL support, X509_V_FLAG_CRL_CHECK and
    /// X509_V_FLAG_CRL_CHECK_ALL are both set, requiring CRL check against all
    /// the elements of the certificate chain if a CRL file is passed.
    ///
    /// This option makes sense only when used in combination with the
    /// `ssl_verify_peer` option.
    ///
    /// A specific error code (`is_ssl_crl_badfile`) is defined with the
    /// option. It is returned when the SSL exchange fails because the CRL file
    /// cannot be loaded. A failure in certificate verification due to a
    /// revocation information found in the CRL does not trigger this specific
    /// error.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_CRLFILE`.
    pub fn crlfile<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_CRLFILE, path.as_ref())
    }

    /// Request SSL certificate information
    ///
    /// Enable libcurl's certificate chain info gatherer. With this enabled,
    /// libcurl will extract lots of information and data about the certificates
    /// in the certificate chain used in the SSL connection.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_CERTINFO`.
    pub fn certinfo(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_CERTINFO, enable as c_long)
    }

    // /// Set pinned public key.
    // ///
    // /// Pass a pointer to a zero terminated string as parameter. The string can
    // /// be the file name of your pinned public key. The file format expected is
    // /// "PEM" or "DER". The string can also be any number of base64 encoded
    // /// sha256 hashes preceded by "sha256//" and separated by ";"
    // ///
    // /// When negotiating a TLS or SSL connection, the server sends a certificate
    // /// indicating its identity. A public key is extracted from this certificate
    // /// and if it does not exactly match the public key provided to this option,
    // /// curl will abort the connection before sending or receiving any data.
    // ///
    // /// By default this option is not set and corresponds to
    // /// `CURLOPT_PINNEDPUBLICKEY`.
    // pub fn pinned_public_key(&mut self, enable: bool) -> Result<(), Error> {
    //     self.setopt_long(curl_sys::CURLOPT_CERTINFO, enable as c_long)
    // }

    /// Specify a source for random data
    ///
    /// The file will be used to read from to seed the random engine for SSL and
    /// more.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_RANDOM_FILE`.
    pub fn random_file<P: AsRef<Path>>(&mut self, p: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_RANDOM_FILE, p.as_ref())
    }

    /// Specify EGD socket path.
    ///
    /// Indicates the path name to the Entropy Gathering Daemon socket. It will
    /// be used to seed the random engine for SSL.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_EGDSOCKET`.
    pub fn egd_socket<P: AsRef<Path>>(&mut self, p: P) -> Result<(), Error> {
        self.setopt_path(curl_sys::CURLOPT_EGDSOCKET, p.as_ref())
    }

    /// Specify ciphers to use for TLS.
    ///
    /// Holds the list of ciphers to use for the SSL connection. The list must
    /// be syntactically correct, it consists of one or more cipher strings
    /// separated by colons. Commas or spaces are also acceptable separators
    /// but colons are normally used, !, - and + can be used as operators.
    ///
    /// For OpenSSL and GnuTLS valid examples of cipher lists include 'RC4-SHA',
    /// ´SHA1+DES´, 'TLSv1' and 'DEFAULT'. The default list is normally set when
    /// you compile OpenSSL.
    ///
    /// You'll find more details about cipher lists on this URL:
    ///
    /// https://www.openssl.org/docs/apps/ciphers.html
    ///
    /// For NSS, valid examples of cipher lists include 'rsa_rc4_128_md5',
    /// ´rsa_aes_128_sha´, etc. With NSS you don't add/remove ciphers. If one
    /// uses this option then all known ciphers are disabled and only those
    /// passed in are enabled.
    ///
    /// You'll find more details about the NSS cipher lists on this URL:
    ///
    /// http://git.fedorahosted.org/cgit/mod_nss.git/plain/docs/mod_nss.html#Directives
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSL_CIPHER_LIST`.
    pub fn ssl_cipher_list(&mut self, ciphers: &str) -> Result<(), Error> {
        let ciphers = CString::new(ciphers)?;
        self.setopt_str(curl_sys::CURLOPT_SSL_CIPHER_LIST, &ciphers)
    }

    /// Enable or disable use of the SSL session-ID cache
    ///
    /// By default all transfers are done using the cache enabled. While nothing
    /// ever should get hurt by attempting to reuse SSL session-IDs, there seem
    /// to be or have been broken SSL implementations in the wild that may
    /// require you to disable this in order for you to succeed.
    ///
    /// This corresponds to the `CURLOPT_SSL_SESSIONID_CACHE` option.
    pub fn ssl_sessionid_cache(&mut self, enable: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_SSL_SESSIONID_CACHE, enable as c_long)
    }

    /// Set SSL behavior options
    ///
    /// Inform libcurl about SSL specific behaviors.
    ///
    /// This corresponds to the `CURLOPT_SSL_OPTIONS` option.
    pub fn ssl_options(&mut self, bits: &SslOpt) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_SSL_OPTIONS, bits.bits)
    }

    // /// Set SSL behavior options for proxies
    // ///
    // /// Inform libcurl about SSL specific behaviors.
    // ///
    // /// This corresponds to the `CURLOPT_PROXY_SSL_OPTIONS` option.
    // pub fn proxy_ssl_options(&mut self, bits: &SslOpt) -> Result<(), Error> {
    //     self.setopt_long(curl_sys::CURLOPT_PROXY_SSL_OPTIONS, bits.bits)
    // }

    // /// Stores a private pointer-sized piece of data.
    // ///
    // /// This can be retrieved through the `private` function and otherwise
    // /// libcurl does not tamper with this value. This corresponds to
    // /// `CURLOPT_PRIVATE` and defaults to 0.
    // pub fn set_private(&mut self, private: usize) -> Result<(), Error> {
    //     self.setopt_ptr(curl_sys::CURLOPT_PRIVATE, private as *const _)
    // }
    //
    // /// Fetches this handle's private pointer-sized piece of data.
    // ///
    // /// This corresponds to `CURLINFO_PRIVATE` and defaults to 0.
    // pub fn private(&mut self) -> Result<usize, Error> {
    //     self.getopt_ptr(curl_sys::CURLINFO_PRIVATE).map(|p| p as usize)
    // }

    // =========================================================================
    // getters

    /// Get info on unmet time conditional
    ///
    /// Returns if the condition provided in the previous request didn't match
    ///
    //// This corresponds to `CURLINFO_CONDITION_UNMET` and may return an error if the
    /// option is not supported
    pub fn time_condition_unmet(&mut self) -> Result<bool, Error> {
        self.getopt_long(curl_sys::CURLINFO_CONDITION_UNMET).map(
            |r| {
                if r == 0 {
                    false
                } else {
                    true
                }
            },
        )
    }

    /// Get the last used URL
    ///
    /// In cases when you've asked libcurl to follow redirects, it may
    /// not be the same value you set with `url`.
    ///
    /// This methods corresponds to the `CURLINFO_EFFECTIVE_URL` option.
    ///
    /// Returns `Ok(None)` if no effective url is listed or `Err` if an error
    /// happens or the underlying bytes aren't valid utf-8.
    pub fn effective_url(&mut self) -> Result<Option<&str>, Error> {
        self.getopt_str(curl_sys::CURLINFO_EFFECTIVE_URL)
    }

    /// Get the last used URL, in bytes
    ///
    /// In cases when you've asked libcurl to follow redirects, it may
    /// not be the same value you set with `url`.
    ///
    /// This methods corresponds to the `CURLINFO_EFFECTIVE_URL` option.
    ///
    /// Returns `Ok(None)` if no effective url is listed or `Err` if an error
    /// happens or the underlying bytes aren't valid utf-8.
    pub fn effective_url_bytes(&mut self) -> Result<Option<&[u8]>, Error> {
        self.getopt_bytes(curl_sys::CURLINFO_EFFECTIVE_URL)
    }

    /// Get the last response code
    ///
    /// The stored value will be zero if no server response code has been
    /// received. Note that a proxy's CONNECT response should be read with
    /// `http_connectcode` and not this.
    ///
    /// Corresponds to `CURLINFO_RESPONSE_CODE` and returns an error if this
    /// option is not supported.
    pub fn response_code(&mut self) -> Result<u32, Error> {
        self.getopt_long(curl_sys::CURLINFO_RESPONSE_CODE)
            .map(|c| c as u32)
    }

    /// Get the CONNECT response code
    ///
    /// Returns the last received HTTP proxy response code to a CONNECT request.
    /// The returned value will be zero if no such response code was available.
    ///
    /// Corresponds to `CURLINFO_HTTP_CONNECTCODE` and returns an error if this
    /// option is not supported.
    pub fn http_connectcode(&mut self) -> Result<u32, Error> {
        self.getopt_long(curl_sys::CURLINFO_HTTP_CONNECTCODE)
            .map(|c| c as u32)
    }

    /// Get the remote time of the retrieved document
    ///
    /// Returns the remote time of the retrieved document (in number of seconds
    /// since 1 Jan 1970 in the GMT/UTC time zone). If you get `None`, it can be
    /// because of many reasons (it might be unknown, the server might hide it
    /// or the server doesn't support the command that tells document time etc)
    /// and the time of the document is unknown.
    ///
    /// Note that you must tell the server to collect this information before
    /// the transfer is made, by using the `filetime` method to
    /// or you will unconditionally get a `None` back.
    ///
    /// This corresponds to `CURLINFO_FILETIME` and may return an error if the
    /// option is not supported
    pub fn filetime(&mut self) -> Result<Option<i64>, Error> {
        self.getopt_long(curl_sys::CURLINFO_FILETIME).map(|r| {
            if r == -1 {
                None
            } else {
                Some(r as i64)
            }
        })
    }

    /// Get the number of downloaded bytes
    ///
    /// Returns the total amount of bytes that were downloaded.
    /// The amount is only for the latest transfer and will be reset again for each new transfer.
    /// This counts actual payload data, what's also commonly called body.
    /// All meta and header data are excluded and will not be counted in this number.
    ///
    /// This corresponds to `CURLINFO_SIZE_DOWNLOAD` and may return an error if the
    /// option is not supported
    pub fn download_size(&mut self) -> Result<f64, Error> {
        self.getopt_double(curl_sys::CURLINFO_SIZE_DOWNLOAD)
            .map(|r| r as f64)
    }

    /// Get the content-length of the download
    ///
    /// Returns the content-length of the download.
    /// This is the value read from the Content-Length: field
    ///
    /// This corresponds to `CURLINFO_CONTENT_LENGTH_DOWNLOAD` and may return an error if the
    /// option is not supported
    pub fn content_length_download(&mut self) -> Result<f64, Error> {
        self.getopt_double(curl_sys::CURLINFO_CONTENT_LENGTH_DOWNLOAD)
            .map(|r| r as f64)
    }

    /// Get total time of previous transfer
    ///
    /// Returns the total time for the previous transfer,
    /// including name resolving, TCP connect etc.
    ///
    /// Corresponds to `CURLINFO_TOTAL_TIME` and may return an error if the
    /// option isn't supported.
    pub fn total_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_TOTAL_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the name lookup time
    ///
    /// Returns the total time from the start
    /// until the name resolving was completed.
    ///
    /// Corresponds to `CURLINFO_NAMELOOKUP_TIME` and may return an error if the
    /// option isn't supported.
    pub fn namelookup_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_NAMELOOKUP_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the time until connect
    ///
    /// Returns the total time from the start
    /// until the connection to the remote host (or proxy) was completed.
    ///
    /// Corresponds to `CURLINFO_CONNECT_TIME` and may return an error if the
    /// option isn't supported.
    pub fn connect_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_CONNECT_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the time until the SSL/SSH handshake is completed
    ///
    /// Returns the total time it took from the start until the SSL/SSH
    /// connect/handshake to the remote host was completed. This time is most often
    /// very near to the `pretransfer_time` time, except for cases such as
    /// HTTP pipelining where the pretransfer time can be delayed due to waits in
    /// line for the pipeline and more.
    ///
    /// Corresponds to `CURLINFO_APPCONNECT_TIME` and may return an error if the
    /// option isn't supported.
    pub fn appconnect_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_APPCONNECT_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the time until the file transfer start
    ///
    /// Returns the total time it took from the start until the file
    /// transfer is just about to begin. This includes all pre-transfer commands
    /// and negotiations that are specific to the particular protocol(s) involved.
    /// It does not involve the sending of the protocol- specific request that
    /// triggers a transfer.
    ///
    /// Corresponds to `CURLINFO_PRETRANSFER_TIME` and may return an error if the
    /// option isn't supported.
    pub fn pretransfer_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_PRETRANSFER_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the time until the first byte is received
    ///
    /// Returns the total time it took from the start until the first
    /// byte is received by libcurl. This includes `pretransfer_time` and
    /// also the time the server needs to calculate the result.
    ///
    /// Corresponds to `CURLINFO_STARTTRANSFER_TIME` and may return an error if the
    /// option isn't supported.
    pub fn starttransfer_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_STARTTRANSFER_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the time for all redirection steps
    ///
    /// Returns the total time it took for all redirection steps
    /// include name lookup, connect, pretransfer and transfer before final
    /// transaction was started. `redirect_time` contains the complete
    /// execution time for multiple redirections.
    ///
    /// Corresponds to `CURLINFO_REDIRECT_TIME` and may return an error if the
    /// option isn't supported.
    pub fn redirect_time(&mut self) -> Result<Duration, Error> {
        self.getopt_double(curl_sys::CURLINFO_REDIRECT_TIME)
            .map(double_seconds_to_duration)
    }

    /// Get the number of redirects
    ///
    /// Corresponds to `CURLINFO_REDIRECT_COUNT` and may return an error if the
    /// option isn't supported.
    pub fn redirect_count(&mut self) -> Result<u32, Error> {
        self.getopt_long(curl_sys::CURLINFO_REDIRECT_COUNT)
            .map(|c| c as u32)
    }

    /// Get the URL a redirect would go to
    ///
    /// Returns the URL a redirect would take you to if you would enable
    /// `follow_location`. This can come very handy if you think using the
    /// built-in libcurl redirect logic isn't good enough for you but you would
    /// still prefer to avoid implementing all the magic of figuring out the new
    /// URL.
    ///
    /// Corresponds to `CURLINFO_REDIRECT_URL` and may return an error if the
    /// url isn't valid utf-8 or an error happens.
    pub fn redirect_url(&mut self) -> Result<Option<&str>, Error> {
        self.getopt_str(curl_sys::CURLINFO_REDIRECT_URL)
    }

    /// Get the URL a redirect would go to, in bytes
    ///
    /// Returns the URL a redirect would take you to if you would enable
    /// `follow_location`. This can come very handy if you think using the
    /// built-in libcurl redirect logic isn't good enough for you but you would
    /// still prefer to avoid implementing all the magic of figuring out the new
    /// URL.
    ///
    /// Corresponds to `CURLINFO_REDIRECT_URL` and may return an error.
    pub fn redirect_url_bytes(&mut self) -> Result<Option<&[u8]>, Error> {
        self.getopt_bytes(curl_sys::CURLINFO_REDIRECT_URL)
    }

    /// Get size of retrieved headers
    ///
    /// Corresponds to `CURLINFO_HEADER_SIZE` and may return an error if the
    /// option isn't supported.
    pub fn header_size(&mut self) -> Result<u64, Error> {
        self.getopt_long(curl_sys::CURLINFO_HEADER_SIZE)
            .map(|c| c as u64)
    }

    /// Get size of sent request.
    ///
    /// Corresponds to `CURLINFO_REQUEST_SIZE` and may return an error if the
    /// option isn't supported.
    pub fn request_size(&mut self) -> Result<u64, Error> {
        self.getopt_long(curl_sys::CURLINFO_REQUEST_SIZE)
            .map(|c| c as u64)
    }

    /// Get Content-Type
    ///
    /// Returns the content-type of the downloaded object. This is the value
    /// read from the Content-Type: field.  If you get `None`, it means that the
    /// server didn't send a valid Content-Type header or that the protocol
    /// used doesn't support this.
    ///
    /// Corresponds to `CURLINFO_CONTENT_TYPE` and may return an error if the
    /// option isn't supported.
    pub fn content_type(&mut self) -> Result<Option<&str>, Error> {
        self.getopt_str(curl_sys::CURLINFO_CONTENT_TYPE)
    }

    /// Get Content-Type, in bytes
    ///
    /// Returns the content-type of the downloaded object. This is the value
    /// read from the Content-Type: field.  If you get `None`, it means that the
    /// server didn't send a valid Content-Type header or that the protocol
    /// used doesn't support this.
    ///
    /// Corresponds to `CURLINFO_CONTENT_TYPE` and may return an error if the
    /// option isn't supported.
    pub fn content_type_bytes(&mut self) -> Result<Option<&[u8]>, Error> {
        self.getopt_bytes(curl_sys::CURLINFO_CONTENT_TYPE)
    }

    /// Get errno number from last connect failure.
    ///
    /// Note that the value is only set on failure, it is not reset upon a
    /// successful operation. The number is OS and system specific.
    ///
    /// Corresponds to `CURLINFO_OS_ERRNO` and may return an error if the
    /// option isn't supported.
    pub fn os_errno(&mut self) -> Result<i32, Error> {
        self.getopt_long(curl_sys::CURLINFO_OS_ERRNO)
            .map(|c| c as i32)
    }

    /// Get IP address of last connection.
    ///
    /// Returns a string holding the IP address of the most recent connection
    /// done with this curl handle. This string may be IPv6 when that is
    /// enabled.
    ///
    /// Corresponds to `CURLINFO_PRIMARY_IP` and may return an error if the
    /// option isn't supported.
    pub fn primary_ip(&mut self) -> Result<Option<&str>, Error> {
        self.getopt_str(curl_sys::CURLINFO_PRIMARY_IP)
    }

    /// Get the latest destination port number
    ///
    /// Corresponds to `CURLINFO_PRIMARY_PORT` and may return an error if the
    /// option isn't supported.
    pub fn primary_port(&mut self) -> Result<u16, Error> {
        self.getopt_long(curl_sys::CURLINFO_PRIMARY_PORT)
            .map(|c| c as u16)
    }

    /// Get local IP address of last connection
    ///
    /// Returns a string holding the IP address of the local end of most recent
    /// connection done with this curl handle. This string may be IPv6 when that
    /// is enabled.
    ///
    /// Corresponds to `CURLINFO_LOCAL_IP` and may return an error if the
    /// option isn't supported.
    pub fn local_ip(&mut self) -> Result<Option<&str>, Error> {
        self.getopt_str(curl_sys::CURLINFO_LOCAL_IP)
    }

    /// Get the latest local port number
    ///
    /// Corresponds to `CURLINFO_LOCAL_PORT` and may return an error if the
    /// option isn't supported.
    pub fn local_port(&mut self) -> Result<u16, Error> {
        self.getopt_long(curl_sys::CURLINFO_LOCAL_PORT)
            .map(|c| c as u16)
    }

    /// Get all known cookies
    ///
    /// Returns a linked-list of all cookies cURL knows (expired ones, too).
    ///
    /// Corresponds to the `CURLINFO_COOKIELIST` option and may return an error
    /// if the option isn't supported.
    pub fn cookies(&mut self) -> Result<List, Error> {
        unsafe {
            let mut list = 0 as *mut _;
            let rc = curl_sys::curl_easy_getinfo(
                self.inner.handle,
                curl_sys::CURLINFO_COOKIELIST,
                &mut list,
            );
            self.cvt(rc)?;
            Ok(list::from_raw(list))
        }
    }

    /// Wait for pipelining/multiplexing
    ///
    /// Set wait to `true` to tell libcurl to prefer to wait for a connection to
    /// confirm or deny that it can do pipelining or multiplexing before
    /// continuing.
    ///
    /// When about to perform a new transfer that allows pipelining or
    /// multiplexing, libcurl will check for existing connections to re-use and
    /// pipeline on. If no such connection exists it will immediately continue
    /// and create a fresh new connection to use.
    ///
    /// By setting this option to `true` - and having `pipelining(true, true)`
    /// enabled for the multi handle this transfer is associated with - libcurl
    /// will instead wait for the connection to reveal if it is possible to
    /// pipeline/multiplex on before it continues. This enables libcurl to much
    /// better keep the number of connections to a minimum when using pipelining
    /// or multiplexing protocols.
    ///
    /// The effect thus becomes that with this option set, libcurl prefers to
    /// wait and re-use an existing connection for pipelining rather than the
    /// opposite: prefer to open a new connection rather than waiting.
    ///
    /// The waiting time is as long as it takes for the connection to get up and
    /// for libcurl to get the necessary response back that informs it about its
    /// protocol and support level.
    ///
    /// This corresponds to the `CURLOPT_PIPEWAIT` option.
    pub fn pipewait(&mut self, wait: bool) -> Result<(), Error> {
        self.setopt_long(curl_sys::CURLOPT_PIPEWAIT, wait as c_long)
    }

    // =========================================================================
    // Other methods

    /// After options have been set, this will perform the transfer described by
    /// the options.
    ///
    /// This performs the request in a synchronous fashion. This can be used
    /// multiple times for one easy handle and libcurl will attempt to re-use
    /// the same connection for all transfers.
    ///
    /// This method will preserve all options configured in this handle for the
    /// next request, and if that is not desired then the options can be
    /// manually reset or the `reset` method can be called.
    ///
    /// Note that this method takes `&self`, which is quite important! This
    /// allows applications to close over the handle in various callbacks to
    /// call methods like `unpause_write` and `unpause_read` while a transfer is
    /// in progress.
    pub fn perform(&self) -> Result<(), Error> {
        let ret = unsafe { self.cvt(curl_sys::curl_easy_perform(self.inner.handle)) };
        panic::propagate();
        return ret;
    }

    /// Unpause reading on a connection.
    ///
    /// Using this function, you can explicitly unpause a connection that was
    /// previously paused.
    ///
    /// A connection can be paused by letting the read or the write callbacks
    /// return `ReadError::Pause` or `WriteError::Pause`.
    ///
    /// To unpause, you may for example call this from the progress callback
    /// which gets called at least once per second, even if the connection is
    /// paused.
    ///
    /// The chance is high that you will get your write callback called before
    /// this function returns.
    pub fn unpause_read(&self) -> Result<(), Error> {
        unsafe {
            let rc = curl_sys::curl_easy_pause(self.inner.handle, curl_sys::CURLPAUSE_RECV_CONT);
            self.cvt(rc)
        }
    }

    /// Unpause writing on a connection.
    ///
    /// Using this function, you can explicitly unpause a connection that was
    /// previously paused.
    ///
    /// A connection can be paused by letting the read or the write callbacks
    /// return `ReadError::Pause` or `WriteError::Pause`. A write callback that
    /// returns pause signals to the library that it couldn't take care of any
    /// data at all, and that data will then be delivered again to the callback
    /// when the writing is later unpaused.
    ///
    /// To unpause, you may for example call this from the progress callback
    /// which gets called at least once per second, even if the connection is
    /// paused.
    pub fn unpause_write(&self) -> Result<(), Error> {
        unsafe {
            let rc = curl_sys::curl_easy_pause(self.inner.handle, curl_sys::CURLPAUSE_SEND_CONT);
            self.cvt(rc)
        }
    }

    /// URL encodes a string `s`
    pub fn url_encode(&mut self, s: &[u8]) -> String {
        if s.len() == 0 {
            return String::new();
        }
        unsafe {
            let p = curl_sys::curl_easy_escape(
                self.inner.handle,
                s.as_ptr() as *const _,
                s.len() as c_int,
            );
            assert!(!p.is_null());
            let ret = str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap();
            let ret = String::from(ret);
            curl_sys::curl_free(p as *mut _);
            return ret;
        }
    }

    /// URL decodes a string `s`, returning `None` if it fails
    pub fn url_decode(&mut self, s: &str) -> Vec<u8> {
        if s.len() == 0 {
            return Vec::new();
        }

        // Work around https://curl.haxx.se/docs/adv_20130622.html, a bug where
        // if the last few characters are a bad escape then curl will have a
        // buffer overrun.
        let mut iter = s.chars().rev();
        let orig_len = s.len();
        let mut data;
        let mut s = s;
        if iter.next() == Some('%') || iter.next() == Some('%') || iter.next() == Some('%') {
            data = s.to_string();
            data.push(0u8 as char);
            s = &data[..];
        }
        unsafe {
            let mut len = 0;
            let p = curl_sys::curl_easy_unescape(
                self.inner.handle,
                s.as_ptr() as *const _,
                orig_len as c_int,
                &mut len,
            );
            assert!(!p.is_null());
            let slice = slice::from_raw_parts(p as *const u8, len as usize);
            let ret = slice.to_vec();
            curl_sys::curl_free(p as *mut _);
            return ret;
        }
    }

    // TODO: I don't think this is safe, you can drop this which has all the
    //       callback data and then the next is use-after-free
    //
    // /// Attempts to clone this handle, returning a new session handle with the
    // /// same options set for this handle.
    // ///
    // /// Internal state info and things like persistent connections ccannot be
    // /// transferred.
    // ///
    // /// # Errors
    // ///
    // /// If a new handle could not be allocated or another error happens, `None`
    // /// is returned.
    // pub fn try_clone<'b>(&mut self) -> Option<Easy<'b>> {
    //     unsafe {
    //         let handle = curl_sys::curl_easy_duphandle(self.handle);
    //         if handle.is_null() {
    //             None
    //         } else {
    //             Some(Easy {
    //                 handle: handle,
    //                 data: blank_data(),
    //                 _marker: marker::PhantomData,
    //             })
    //         }
    //     }
    // }

    /// Receives data from a connected socket.
    ///
    /// Only useful after a successful `perform` with the `connect_only` option
    /// set as well.
    pub fn recv(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        unsafe {
            let mut n = 0;
            let r = curl_sys::curl_easy_recv(
                self.inner.handle,
                data.as_mut_ptr() as *mut _,
                data.len(),
                &mut n,
            );
            if r == curl_sys::CURLE_OK {
                Ok(n)
            } else {
                Err(Error::new(r))
            }
        }
    }

    /// Sends data over the connected socket.
    ///
    /// Only useful after a successful `perform` with the `connect_only` option
    /// set as well.
    pub fn send(&mut self, data: &[u8]) -> Result<usize, Error> {
        unsafe {
            let mut n = 0;
            let rc = curl_sys::curl_easy_send(
                self.inner.handle,
                data.as_ptr() as *const _,
                data.len(),
                &mut n,
            );
            self.cvt(rc)?;
            Ok(n)
        }
    }

    /// Get a pointer to the raw underlying CURL handle.
    pub fn raw(&self) -> *mut curl_sys::CURL {
        self.inner.handle
    }

    #[cfg(unix)]
    fn setopt_path(&mut self, opt: curl_sys::CURLoption, val: &Path) -> Result<(), Error> {
        use std::os::unix::prelude::*;
        let s = CString::new(val.as_os_str().as_bytes())?;
        self.setopt_str(opt, &s)
    }

    #[cfg(windows)]
    fn setopt_path(&mut self, opt: curl_sys::CURLoption, val: &Path) -> Result<(), Error> {
        match val.to_str() {
            Some(s) => self.setopt_str(opt, &CString::new(s)?),
            None => Err(Error::new(curl_sys::CURLE_CONV_FAILED)),
        }
    }

    fn setopt_long(&mut self, opt: curl_sys::CURLoption, val: c_long) -> Result<(), Error> {
        unsafe { self.cvt(curl_sys::curl_easy_setopt(self.inner.handle, opt, val)) }
    }

    fn setopt_str(&mut self, opt: curl_sys::CURLoption, val: &CStr) -> Result<(), Error> {
        self.setopt_ptr(opt, val.as_ptr())
    }

    fn setopt_ptr(&self, opt: curl_sys::CURLoption, val: *const c_char) -> Result<(), Error> {
        unsafe { self.cvt(curl_sys::curl_easy_setopt(self.inner.handle, opt, val)) }
    }

    fn setopt_off_t(
        &mut self,
        opt: curl_sys::CURLoption,
        val: curl_sys::curl_off_t,
    ) -> Result<(), Error> {
        unsafe {
            let rc = curl_sys::curl_easy_setopt(self.inner.handle, opt, val);
            self.cvt(rc)
        }
    }

    fn getopt_bytes(&mut self, opt: curl_sys::CURLINFO) -> Result<Option<&[u8]>, Error> {
        unsafe {
            let p = self.getopt_ptr(opt)?;
            if p.is_null() {
                Ok(None)
            } else {
                Ok(Some(CStr::from_ptr(p).to_bytes()))
            }
        }
    }

    fn getopt_ptr(&mut self, opt: curl_sys::CURLINFO) -> Result<*const c_char, Error> {
        unsafe {
            let mut p = 0 as *const c_char;
            let rc = curl_sys::curl_easy_getinfo(self.inner.handle, opt, &mut p);
            self.cvt(rc)?;
            Ok(p)
        }
    }

    fn getopt_str(&mut self, opt: curl_sys::CURLINFO) -> Result<Option<&str>, Error> {
        match self.getopt_bytes(opt) {
            Ok(None) => Ok(None),
            Err(e) => Err(e),
            Ok(Some(bytes)) => match str::from_utf8(bytes) {
                Ok(s) => Ok(Some(s)),
                Err(_) => Err(Error::new(curl_sys::CURLE_CONV_FAILED)),
            },
        }
    }

    fn getopt_long(&mut self, opt: curl_sys::CURLINFO) -> Result<c_long, Error> {
        unsafe {
            let mut p = 0;
            let rc = curl_sys::curl_easy_getinfo(self.inner.handle, opt, &mut p);
            self.cvt(rc)?;
            Ok(p)
        }
    }

    fn getopt_double(&mut self, opt: curl_sys::CURLINFO) -> Result<c_double, Error> {
        unsafe {
            let mut p = 0 as c_double;
            let rc = curl_sys::curl_easy_getinfo(self.inner.handle, opt, &mut p);
            self.cvt(rc)?;
            Ok(p)
        }
    }

    /// Returns the contents of the internal error buffer, if available.
    ///
    /// When an easy handle is created it configured the `CURLOPT_ERRORBUFFER`
    /// parameter and instructs libcurl to store more error information into a
    /// buffer for better error messages and better debugging. The contents of
    /// that buffer are automatically coupled with all errors for methods on
    /// this type, but if manually invoking APIs the contents will need to be
    /// extracted with this method.
    ///
    /// Put another way, you probably don't need this, you're probably already
    /// getting nice error messages!
    ///
    /// This function will clear the internal buffer, so this is an operation
    /// that mutates the handle internally.
    pub fn take_error_buf(&self) -> Option<String> {
        let mut buf = self.inner.error_buf.borrow_mut();
        if buf[0] == 0 {
            return None;
        }
        let pos = buf.iter().position(|i| *i == 0).unwrap_or(buf.len());
        let msg = String::from_utf8_lossy(&buf[..pos]).into_owned();
        buf[0] = 0;
        Some(msg)
    }

    fn cvt(&self, rc: curl_sys::CURLcode) -> Result<(), Error> {
        if rc == curl_sys::CURLE_OK {
            return Ok(());
        }
        let mut err = Error::new(rc);
        if let Some(msg) = self.take_error_buf() {
            err.set_extra(msg);
        }
        Err(Error::new(rc))
    }
}

impl<H: fmt::Debug> fmt::Debug for Easy2<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Easy")
            .field("handle", &self.inner.handle)
            .field("handler", &self.inner.handle)
            .finish()
    }
}

impl<H> Drop for Easy2<H> {
    fn drop(&mut self) {
        unsafe {
            curl_sys::curl_easy_cleanup(self.inner.handle);
        }
    }
}

extern "C" fn header_cb<H: Handler>(
    buffer: *mut c_char,
    size: size_t,
    nitems: size_t,
    userptr: *mut c_void,
) -> size_t {
    let keep_going = panic::catch(|| unsafe {
        let data = slice::from_raw_parts(buffer as *const u8, size * nitems);
        (*(userptr as *mut Inner<H>)).handler.header(data)
    })
    .unwrap_or(false);
    if keep_going {
        size * nitems
    } else {
        !0
    }
}

extern "C" fn write_cb<H: Handler>(
    ptr: *mut c_char,
    size: size_t,
    nmemb: size_t,
    data: *mut c_void,
) -> size_t {
    panic::catch(|| unsafe {
        let input = slice::from_raw_parts(ptr as *const u8, size * nmemb);
        match (*(data as *mut Inner<H>)).handler.write(input) {
            Ok(s) => s,
            Err(WriteError::Pause) | Err(WriteError::__Nonexhaustive) => {
                curl_sys::CURL_WRITEFUNC_PAUSE
            }
        }
    })
    .unwrap_or(!0)
}

extern "C" fn read_cb<H: Handler>(
    ptr: *mut c_char,
    size: size_t,
    nmemb: size_t,
    data: *mut c_void,
) -> size_t {
    panic::catch(|| unsafe {
        let input = slice::from_raw_parts_mut(ptr as *mut u8, size * nmemb);
        match (*(data as *mut Inner<H>)).handler.read(input) {
            Ok(s) => s,
            Err(ReadError::Pause) => curl_sys::CURL_READFUNC_PAUSE,
            Err(ReadError::__Nonexhaustive) | Err(ReadError::Abort) => {
                curl_sys::CURL_READFUNC_ABORT
            }
        }
    })
    .unwrap_or(!0)
}

extern "C" fn seek_cb<H: Handler>(
    data: *mut c_void,
    offset: curl_sys::curl_off_t,
    origin: c_int,
) -> c_int {
    panic::catch(|| unsafe {
        let from = if origin == libc::SEEK_SET {
            SeekFrom::Start(offset as u64)
        } else {
            panic!("unknown origin from libcurl: {}", origin);
        };
        (*(data as *mut Inner<H>)).handler.seek(from) as c_int
    })
    .unwrap_or(!0)
}

extern "C" fn progress_cb<H: Handler>(
    data: *mut c_void,
    dltotal: c_double,
    dlnow: c_double,
    ultotal: c_double,
    ulnow: c_double,
) -> c_int {
    let keep_going = panic::catch(|| unsafe {
        (*(data as *mut Inner<H>))
            .handler
            .progress(dltotal, dlnow, ultotal, ulnow)
    })
    .unwrap_or(false);
    if keep_going {
        0
    } else {
        1
    }
}

// TODO: expose `handle`? is that safe?
extern "C" fn debug_cb<H: Handler>(
    _handle: *mut curl_sys::CURL,
    kind: curl_sys::curl_infotype,
    data: *mut c_char,
    size: size_t,
    userptr: *mut c_void,
) -> c_int {
    panic::catch(|| unsafe {
        let data = slice::from_raw_parts(data as *const u8, size);
        let kind = match kind {
            curl_sys::CURLINFO_TEXT => InfoType::Text,
            curl_sys::CURLINFO_HEADER_IN => InfoType::HeaderIn,
            curl_sys::CURLINFO_HEADER_OUT => InfoType::HeaderOut,
            curl_sys::CURLINFO_DATA_IN => InfoType::DataIn,
            curl_sys::CURLINFO_DATA_OUT => InfoType::DataOut,
            curl_sys::CURLINFO_SSL_DATA_IN => InfoType::SslDataIn,
            curl_sys::CURLINFO_SSL_DATA_OUT => InfoType::SslDataOut,
            _ => return,
        };
        (*(userptr as *mut Inner<H>)).handler.debug(kind, data)
    });
    return 0;
}

extern "C" fn ssl_ctx_cb<H: Handler>(
    _handle: *mut curl_sys::CURL,
    ssl_ctx: *mut c_void,
    data: *mut c_void,
) -> curl_sys::CURLcode {
    let res = panic::catch(|| unsafe {
        match (*(data as *mut Inner<H>)).handler.ssl_ctx(ssl_ctx) {
            Ok(()) => curl_sys::CURLE_OK,
            Err(e) => e.code(),
        }
    });
    // Default to a generic SSL error in case of panic. This
    // shouldn't really matter since the error should be
    // propagated later on but better safe than sorry...
    res.unwrap_or(curl_sys::CURLE_SSL_CONNECT_ERROR)
}

// TODO: expose `purpose` and `sockaddr` inside of `address`
extern "C" fn opensocket_cb<H: Handler>(
    data: *mut c_void,
    _purpose: curl_sys::curlsocktype,
    address: *mut curl_sys::curl_sockaddr,
) -> curl_sys::curl_socket_t {
    let res = panic::catch(|| unsafe {
        (*(data as *mut Inner<H>))
            .handler
            .open_socket((*address).family, (*address).socktype, (*address).protocol)
            .unwrap_or(curl_sys::CURL_SOCKET_BAD)
    });
    res.unwrap_or(curl_sys::CURL_SOCKET_BAD)
}

fn double_seconds_to_duration(seconds: f64) -> Duration {
    let whole_seconds = seconds.trunc() as u64;
    let nanos = seconds.fract() * 1_000_000_000f64;
    Duration::new(whole_seconds, nanos as u32)
}

#[test]
fn double_seconds_to_duration_whole_second() {
    let dur = double_seconds_to_duration(1.0);
    assert_eq!(dur.as_secs(), 1);
    assert_eq!(dur.subsec_nanos(), 0);
}

#[test]
fn double_seconds_to_duration_sub_second1() {
    let dur = double_seconds_to_duration(0.0);
    assert_eq!(dur.as_secs(), 0);
    assert_eq!(dur.subsec_nanos(), 0);
}

#[test]
fn double_seconds_to_duration_sub_second2() {
    let dur = double_seconds_to_duration(0.5);
    assert_eq!(dur.as_secs(), 0);
    assert_eq!(dur.subsec_nanos(), 500_000_000);
}

impl Auth {
    /// Creates a new set of authentications with no members.
    ///
    /// An `Auth` structure is used to configure which forms of authentication
    /// are attempted when negotiating connections with servers.
    pub fn new() -> Auth {
        Auth { bits: 0 }
    }

    /// HTTP Basic authentication.
    ///
    /// This is the default choice, and the only method that is in wide-spread
    /// use and supported virtually everywhere.  This sends the user name and
    /// password over the network in plain text, easily captured by others.
    pub fn basic(&mut self, on: bool) -> &mut Auth {
        self.flag(curl_sys::CURLAUTH_BASIC, on)
    }

    /// HTTP Digest authentication.
    ///
    /// Digest authentication is defined in RFC 2617 and is a more secure way to
    /// do authentication over public networks than the regular old-fashioned
    /// Basic method.
    pub fn digest(&mut self, on: bool) -> &mut Auth {
        self.flag(curl_sys::CURLAUTH_DIGEST, on)
    }

    /// HTTP Digest authentication with an IE flavor.
    ///
    /// Digest authentication is defined in RFC 2617 and is a more secure way to
    /// do authentication over public networks than the regular old-fashioned
    /// Basic method. The IE flavor is simply that libcurl will use a special
    /// "quirk" that IE is known to have used before version 7 and that some
    /// servers require the client to use.
    pub fn digest_ie(&mut self, on: bool) -> &mut Auth {
        self.flag(curl_sys::CURLAUTH_DIGEST_IE, on)
    }

    /// HTTP Negotiate (SPNEGO) authentication.
    ///
    /// Negotiate authentication is defined in RFC 4559 and is the most secure
    /// way to perform authentication over HTTP.
    ///
    /// You need to build libcurl with a suitable GSS-API library or SSPI on
    /// Windows for this to work.
    pub fn gssnegotiate(&mut self, on: bool) -> &mut Auth {
        self.flag(curl_sys::CURLAUTH_GSSNEGOTIATE, on)
    }

    /// HTTP NTLM authentication.
    ///
    /// A proprietary protocol invented and used by Microsoft. It uses a
    /// challenge-response and hash concept similar to Digest, to prevent the
    /// password from being eavesdropped.
    ///
    /// You need to build libcurl with either OpenSSL, GnuTLS or NSS support for
    /// this option to work, or build libcurl on Windows with SSPI support.
    pub fn ntlm(&mut self, on: bool) -> &mut Auth {
        self.flag(curl_sys::CURLAUTH_NTLM, on)
    }

    /// NTLM delegating to winbind helper.
    ///
    /// Authentication is performed by a separate binary application that is
    /// executed when needed. The name of the application is specified at
    /// compile time but is typically /usr/bin/ntlm_auth
    ///
    /// Note that libcurl will fork when necessary to run the winbind
    /// application and kill it when complete, calling waitpid() to await its
    /// exit when done. On POSIX operating systems, killing the process will
    /// cause a SIGCHLD signal to be raised (regardless of whether
    /// CURLOPT_NOSIGNAL is set), which must be handled intelligently by the
    /// application. In particular, the application must not unconditionally
    /// call wait() in its SIGCHLD signal handler to avoid being subject to a
    /// race condition. This behavior is subject to change in future versions of
    /// libcurl.
    ///
    /// A proprietary protocol invented and used by Microsoft. It uses a
    /// challenge-response and hash concept similar to Digest, to prevent the
    /// password from being eavesdropped.
    pub fn ntlm_wb(&mut self, on: bool) -> &mut Auth {
        self.flag(curl_sys::CURLAUTH_NTLM_WB, on)
    }

    fn flag(&mut self, bit: c_ulong, on: bool) -> &mut Auth {
        if on {
            self.bits |= bit as c_long;
        } else {
            self.bits &= !bit as c_long;
        }
        self
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bits = self.bits as c_ulong;
        f.debug_struct("Auth")
            .field("basic", &(bits & curl_sys::CURLAUTH_BASIC != 0))
            .field("digest", &(bits & curl_sys::CURLAUTH_DIGEST != 0))
            .field("digest_ie", &(bits & curl_sys::CURLAUTH_DIGEST_IE != 0))
            .field(
                "gssnegotiate",
                &(bits & curl_sys::CURLAUTH_GSSNEGOTIATE != 0),
            )
            .field("ntlm", &(bits & curl_sys::CURLAUTH_NTLM != 0))
            .field("ntlm_wb", &(bits & curl_sys::CURLAUTH_NTLM_WB != 0))
            .finish()
    }
}

impl SslOpt {
    /// Creates a new set of SSL options.
    pub fn new() -> SslOpt {
        SslOpt { bits: 0 }
    }

    /// Tells libcurl to disable certificate revocation checks for those SSL
    /// backends where such behavior is present.
    ///
    /// Currently this option is only supported for WinSSL (the native Windows
    /// SSL library), with an exception in the case of Windows' Untrusted
    /// Publishers blacklist which it seems can't be bypassed. This option may
    /// have broader support to accommodate other SSL backends in the future.
    /// https://curl.haxx.se/docs/ssl-compared.html
    pub fn no_revoke(&mut self, on: bool) -> &mut SslOpt {
        self.flag(curl_sys::CURLSSLOPT_NO_REVOKE, on)
    }

    /// Tells libcurl to not attempt to use any workarounds for a security flaw
    /// in the SSL3 and TLS1.0 protocols.
    ///
    /// If this option isn't used or this bit is set to 0, the SSL layer libcurl
    /// uses may use a work-around for this flaw although it might cause
    /// interoperability problems with some (older) SSL implementations.
    ///
    /// > WARNING: avoiding this work-around lessens the security, and by
    /// > setting this option to 1 you ask for exactly that. This option is only
    /// > supported for DarwinSSL, NSS and OpenSSL.
    pub fn allow_beast(&mut self, on: bool) -> &mut SslOpt {
        self.flag(curl_sys::CURLSSLOPT_ALLOW_BEAST, on)
    }

    fn flag(&mut self, bit: c_long, on: bool) -> &mut SslOpt {
        if on {
            self.bits |= bit as c_long;
        } else {
            self.bits &= !bit as c_long;
        }
        self
    }
}

impl fmt::Debug for SslOpt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SslOpt")
            .field(
                "no_revoke",
                &(self.bits & curl_sys::CURLSSLOPT_NO_REVOKE != 0),
            )
            .field(
                "allow_beast",
                &(self.bits & curl_sys::CURLSSLOPT_ALLOW_BEAST != 0),
            )
            .finish()
    }
}
