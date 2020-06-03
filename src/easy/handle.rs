use std::cell::Cell;
use std::fmt;
use std::io::SeekFrom;
use std::path::Path;
use std::ptr;
use std::str;
use std::time::Duration;

use curl_sys;
use libc::c_void;

use easy::handler::{self, InfoType, ReadError, SeekResult, WriteError};
use easy::handler::{Auth, NetRc, ProxyType, SslOpt};
use easy::handler::{HttpVersion, IpResolve, SslVersion, TimeCondition};
use easy::{Easy2, Handler};
use easy::{Form, List};
use Error;

/// Raw bindings to a libcurl "easy session".
///
/// This type is the same as the `Easy2` type in this library except that it
/// does not contain a type parameter. Callbacks from curl are all controlled
/// via closures on this `Easy` type, and this type namely has a `transfer`
/// method as well for ergonomic management of these callbacks.
///
/// There's not necessarily a right answer for which type is correct to use, but
/// as a general rule of thumb `Easy` is typically a reasonable choice for
/// synchronous I/O and `Easy2` is a good choice for asynchronous I/O.
///
/// ## Examples
///
/// Creating a handle which can be used later
///
/// ```
/// use curl::easy::Easy;
///
/// let handle = Easy::new();
/// ```
///
/// Send an HTTP request, writing the response to stdout.
///
/// ```
/// use std::io::{stdout, Write};
///
/// use curl::easy::Easy;
///
/// let mut handle = Easy::new();
/// handle.url("https://www.rust-lang.org/").unwrap();
/// handle.write_function(|data| {
///     stdout().write_all(data).unwrap();
///     Ok(data.len())
/// }).unwrap();
/// handle.perform().unwrap();
/// ```
///
/// Collect all output of an HTTP request to a vector.
///
/// ```
/// use curl::easy::Easy;
///
/// let mut data = Vec::new();
/// let mut handle = Easy::new();
/// handle.url("https://www.rust-lang.org/").unwrap();
/// {
///     let mut transfer = handle.transfer();
///     transfer.write_function(|new_data| {
///         data.extend_from_slice(new_data);
///         Ok(new_data.len())
///     }).unwrap();
///     transfer.perform().unwrap();
/// }
/// println!("{:?}", data);
/// ```
///
/// More examples of various properties of an HTTP request can be found on the
/// specific methods as well.
#[derive(Debug)]
pub struct Easy {
    inner: Easy2<EasyData>,
}

/// A scoped transfer of information which borrows an `Easy` and allows
/// referencing stack-local data of the lifetime `'data`.
///
/// Usage of `Easy` requires the `'static` and `Send` bounds on all callbacks
/// registered, but that's not often wanted if all you need is to collect a
/// bunch of data in memory to a vector, for example. The `Transfer` structure,
/// created by the `Easy::transfer` method, is used for this sort of request.
///
/// The callbacks attached to a `Transfer` are only active for that one transfer
/// object, and they allow to elide both the `Send` and `'static` bounds to
/// close over stack-local information.
pub struct Transfer<'easy, 'data> {
    easy: &'easy mut Easy,
    data: Box<Callbacks<'data>>,
}

pub struct EasyData {
    running: Cell<bool>,
    owned: Callbacks<'static>,
    borrowed: Cell<*mut Callbacks<'static>>,
}

unsafe impl Send for EasyData {}

#[derive(Default)]
struct Callbacks<'a> {
    write: Option<Box<dyn FnMut(&[u8]) -> Result<usize, WriteError> + 'a>>,
    read: Option<Box<dyn FnMut(&mut [u8]) -> Result<usize, ReadError> + 'a>>,
    seek: Option<Box<dyn FnMut(SeekFrom) -> SeekResult + 'a>>,
    debug: Option<Box<dyn FnMut(InfoType, &[u8]) + 'a>>,
    header: Option<Box<dyn FnMut(&[u8]) -> bool + 'a>>,
    progress: Option<Box<dyn FnMut(f64, f64, f64, f64) -> bool + 'a>>,
    ssl_ctx: Option<Box<dyn FnMut(*mut c_void) -> Result<(), Error> + 'a>>,
}

impl Easy {
    /// Creates a new "easy" handle which is the core of almost all operations
    /// in libcurl.
    ///
    /// To use a handle, applications typically configure a number of options
    /// followed by a call to `perform`. Options are preserved across calls to
    /// `perform` and need to be reset manually (or via the `reset` method) if
    /// this is not desired.
    pub fn new() -> Easy {
        Easy {
            inner: Easy2::new(EasyData {
                running: Cell::new(false),
                owned: Callbacks::default(),
                borrowed: Cell::new(ptr::null_mut()),
            }),
        }
    }

    // =========================================================================
    // Behavior options

    /// Same as [`Easy2::verbose`](struct.Easy2.html#method.verbose)
    pub fn verbose(&mut self, verbose: bool) -> Result<(), Error> {
        self.inner.verbose(verbose)
    }

    /// Same as [`Easy2::show_header`](struct.Easy2.html#method.show_header)
    pub fn show_header(&mut self, show: bool) -> Result<(), Error> {
        self.inner.show_header(show)
    }

    /// Same as [`Easy2::progress`](struct.Easy2.html#method.progress)
    pub fn progress(&mut self, progress: bool) -> Result<(), Error> {
        self.inner.progress(progress)
    }

    /// Same as [`Easy2::signal`](struct.Easy2.html#method.signal)
    pub fn signal(&mut self, signal: bool) -> Result<(), Error> {
        self.inner.signal(signal)
    }

    /// Same as [`Easy2::wildcard_match`](struct.Easy2.html#method.wildcard_match)
    pub fn wildcard_match(&mut self, m: bool) -> Result<(), Error> {
        self.inner.wildcard_match(m)
    }

    /// Same as [`Easy2::unix_socket`](struct.Easy2.html#method.unix_socket)
    pub fn unix_socket(&mut self, unix_domain_socket: &str) -> Result<(), Error> {
        self.inner.unix_socket(unix_domain_socket)
    }

    // =========================================================================
    // Callback options

    /// Set callback for writing received data.
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
    ///
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `write_function` to configure a
    /// callback that can reference stack-local data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{stdout, Write};
    /// use curl::easy::Easy;
    ///
    /// let mut handle = Easy::new();
    /// handle.url("https://www.rust-lang.org/").unwrap();
    /// handle.write_function(|data| {
    ///     Ok(stdout().write(data).unwrap())
    /// }).unwrap();
    /// handle.perform().unwrap();
    /// ```
    ///
    /// Writing to a stack-local buffer
    ///
    /// ```
    /// use std::io::{stdout, Write};
    /// use curl::easy::Easy;
    ///
    /// let mut buf = Vec::new();
    /// let mut handle = Easy::new();
    /// handle.url("https://www.rust-lang.org/").unwrap();
    ///
    /// let mut transfer = handle.transfer();
    /// transfer.write_function(|data| {
    ///     buf.extend_from_slice(data);
    ///     Ok(data.len())
    /// }).unwrap();
    /// transfer.perform().unwrap();
    /// ```
    pub fn write_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> Result<usize, WriteError> + Send + 'static,
    {
        self.inner.get_mut().owned.write = Some(Box::new(f));
        Ok(())
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
    ///
    /// # Examples
    ///
    /// Read input from stdin
    ///
    /// ```no_run
    /// use std::io::{stdin, Read};
    /// use curl::easy::Easy;
    ///
    /// let mut handle = Easy::new();
    /// handle.url("https://example.com/login").unwrap();
    /// handle.read_function(|into| {
    ///     Ok(stdin().read(into).unwrap())
    /// }).unwrap();
    /// handle.post(true).unwrap();
    /// handle.perform().unwrap();
    /// ```
    ///
    /// Reading from stack-local data:
    ///
    /// ```no_run
    /// use std::io::{stdin, Read};
    /// use curl::easy::Easy;
    ///
    /// let mut data_to_upload = &b"foobar"[..];
    /// let mut handle = Easy::new();
    /// handle.url("https://example.com/login").unwrap();
    /// handle.post(true).unwrap();
    ///
    /// let mut transfer = handle.transfer();
    /// transfer.read_function(|into| {
    ///     Ok(data_to_upload.read(into).unwrap())
    /// }).unwrap();
    /// transfer.perform().unwrap();
    /// ```
    pub fn read_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(&mut [u8]) -> Result<usize, ReadError> + Send + 'static,
    {
        self.inner.get_mut().owned.read = Some(Box::new(f));
        Ok(())
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
    ///
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `seek_function` to configure a
    /// callback that can reference stack-local data.
    pub fn seek_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(SeekFrom) -> SeekResult + Send + 'static,
    {
        self.inner.get_mut().owned.seek = Some(Box::new(f));
        Ok(())
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
    ///
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `progress_function` to configure a
    /// callback that can reference stack-local data.
    pub fn progress_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(f64, f64, f64, f64) -> bool + Send + 'static,
    {
        self.inner.get_mut().owned.progress = Some(Box::new(f));
        Ok(())
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
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `progress_function` to configure a
    /// callback that can reference stack-local data.
    pub fn ssl_ctx_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(*mut c_void) -> Result<(), Error> + Send + 'static,
    {
        self.inner.get_mut().owned.ssl_ctx = Some(Box::new(f));
        Ok(())
    }

    /// Specify a debug callback
    ///
    /// `debug_function` replaces the standard debug function used when
    /// `verbose` is in effect. This callback receives debug information,
    /// as specified in the type argument.
    ///
    /// By default this option is not set and corresponds to the
    /// `CURLOPT_DEBUGFUNCTION` and `CURLOPT_DEBUGDATA` options.
    ///
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `debug_function` to configure a
    /// callback that can reference stack-local data.
    pub fn debug_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(InfoType, &[u8]) + Send + 'static,
    {
        self.inner.get_mut().owned.debug = Some(Box::new(f));
        Ok(())
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
    ///
    /// Note that the lifetime bound on this function is `'static`, but that
    /// is often too restrictive. To use stack data consider calling the
    /// `transfer` method and then using `header_function` to configure a
    /// callback that can reference stack-local data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str;
    ///
    /// use curl::easy::Easy;
    ///
    /// let mut handle = Easy::new();
    /// handle.url("https://www.rust-lang.org/").unwrap();
    /// handle.header_function(|header| {
    ///     print!("header: {}", str::from_utf8(header).unwrap());
    ///     true
    /// }).unwrap();
    /// handle.perform().unwrap();
    /// ```
    ///
    /// Collecting headers to a stack local vector
    ///
    /// ```
    /// use std::str;
    ///
    /// use curl::easy::Easy;
    ///
    /// let mut headers = Vec::new();
    /// let mut handle = Easy::new();
    /// handle.url("https://www.rust-lang.org/").unwrap();
    ///
    /// {
    ///     let mut transfer = handle.transfer();
    ///     transfer.header_function(|header| {
    ///         headers.push(str::from_utf8(header).unwrap().to_string());
    ///         true
    ///     }).unwrap();
    ///     transfer.perform().unwrap();
    /// }
    ///
    /// println!("{:?}", headers);
    /// ```
    pub fn header_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> bool + Send + 'static,
    {
        self.inner.get_mut().owned.header = Some(Box::new(f));
        Ok(())
    }

    // =========================================================================
    // Error options

    // TODO: error buffer and stderr

    /// Same as [`Easy2::fail_on_error`](struct.Easy2.html#method.fail_on_error)
    pub fn fail_on_error(&mut self, fail: bool) -> Result<(), Error> {
        self.inner.fail_on_error(fail)
    }

    // =========================================================================
    // Network options

    /// Same as [`Easy2::url`](struct.Easy2.html#method.url)
    pub fn url(&mut self, url: &str) -> Result<(), Error> {
        self.inner.url(url)
    }

    /// Same as [`Easy2::port`](struct.Easy2.html#method.port)
    pub fn port(&mut self, port: u16) -> Result<(), Error> {
        self.inner.port(port)
    }

    /// Same as [`Easy2::proxy`](struct.Easy2.html#method.proxy)
    pub fn proxy(&mut self, url: &str) -> Result<(), Error> {
        self.inner.proxy(url)
    }

    /// Same as [`Easy2::proxy_port`](struct.Easy2.html#method.proxy_port)
    pub fn proxy_port(&mut self, port: u16) -> Result<(), Error> {
        self.inner.proxy_port(port)
    }

    /// Same as [`Easy2::proxy_cainfo`](struct.Easy2.html#method.proxy_cainfo)
    pub fn proxy_cainfo(&mut self, cainfo: &str) -> Result<(), Error> {
        self.inner.proxy_cainfo(cainfo)
    }

    /// Same as [`Easy2::proxy_sslcert`](struct.Easy2.html#method.proxy_sslcert)
    pub fn proxy_sslcert(&mut self, sslcert: &str) -> Result<(), Error> {
        self.inner.proxy_sslcert(sslcert)
    }

    /// Same as [`Easy2::proxy_sslkey`](struct.Easy2.html#method.proxy_sslkey)
    pub fn proxy_sslkey(&mut self, sslkey: &str) -> Result<(), Error> {
        self.inner.proxy_sslkey(sslkey)
    }

    /// Same as [`Easy2::proxy_type`](struct.Easy2.html#method.proxy_type)
    pub fn proxy_type(&mut self, kind: ProxyType) -> Result<(), Error> {
        self.inner.proxy_type(kind)
    }

    /// Same as [`Easy2::noproxy`](struct.Easy2.html#method.noproxy)
    pub fn noproxy(&mut self, skip: &str) -> Result<(), Error> {
        self.inner.noproxy(skip)
    }

    /// Same as [`Easy2::http_proxy_tunnel`](struct.Easy2.html#method.http_proxy_tunnel)
    pub fn http_proxy_tunnel(&mut self, tunnel: bool) -> Result<(), Error> {
        self.inner.http_proxy_tunnel(tunnel)
    }

    /// Same as [`Easy2::interface`](struct.Easy2.html#method.interface)
    pub fn interface(&mut self, interface: &str) -> Result<(), Error> {
        self.inner.interface(interface)
    }

    /// Same as [`Easy2::set_local_port`](struct.Easy2.html#method.set_local_port)
    pub fn set_local_port(&mut self, port: u16) -> Result<(), Error> {
        self.inner.set_local_port(port)
    }

    /// Same as [`Easy2::local_port_range`](struct.Easy2.html#method.local_port_range)
    pub fn local_port_range(&mut self, range: u16) -> Result<(), Error> {
        self.inner.local_port_range(range)
    }

    /// Same as [`Easy2::dns_servers`](struct.Easy2.html#method.dns_servers)
    pub fn dns_servers(&mut self, servers: &str) -> Result<(), Error> {
        self.inner.dns_servers(servers)
    }

    /// Same as [`Easy2::dns_cache_timeout`](struct.Easy2.html#method.dns_cache_timeout)
    pub fn dns_cache_timeout(&mut self, dur: Duration) -> Result<(), Error> {
        self.inner.dns_cache_timeout(dur)
    }

    /// Same as [`Easy2::buffer_size`](struct.Easy2.html#method.buffer_size)
    pub fn buffer_size(&mut self, size: usize) -> Result<(), Error> {
        self.inner.buffer_size(size)
    }

    /// Same as [`Easy2::tcp_nodelay`](struct.Easy2.html#method.tcp_nodelay)
    pub fn tcp_nodelay(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.tcp_nodelay(enable)
    }

    /// Same as [`Easy2::tcp_keepalive`](struct.Easy2.html#method.tcp_keepalive)
    pub fn tcp_keepalive(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.tcp_keepalive(enable)
    }

    /// Same as [`Easy2::tcp_keepintvl`](struct.Easy2.html#method.tcp_keepalive)
    pub fn tcp_keepintvl(&mut self, dur: Duration) -> Result<(), Error> {
        self.inner.tcp_keepintvl(dur)
    }

    /// Same as [`Easy2::tcp_keepidle`](struct.Easy2.html#method.tcp_keepidle)
    pub fn tcp_keepidle(&mut self, dur: Duration) -> Result<(), Error> {
        self.inner.tcp_keepidle(dur)
    }

    /// Same as [`Easy2::address_scope`](struct.Easy2.html#method.address_scope)
    pub fn address_scope(&mut self, scope: u32) -> Result<(), Error> {
        self.inner.address_scope(scope)
    }

    // =========================================================================
    // Names and passwords

    /// Same as [`Easy2::username`](struct.Easy2.html#method.username)
    pub fn username(&mut self, user: &str) -> Result<(), Error> {
        self.inner.username(user)
    }

    /// Same as [`Easy2::password`](struct.Easy2.html#method.password)
    pub fn password(&mut self, pass: &str) -> Result<(), Error> {
        self.inner.password(pass)
    }

    /// Same as [`Easy2::http_auth`](struct.Easy2.html#method.http_auth)
    pub fn http_auth(&mut self, auth: &Auth) -> Result<(), Error> {
        self.inner.http_auth(auth)
    }

    /// Same as [`Easy2::proxy_username`](struct.Easy2.html#method.proxy_username)
    pub fn proxy_username(&mut self, user: &str) -> Result<(), Error> {
        self.inner.proxy_username(user)
    }

    /// Same as [`Easy2::proxy_password`](struct.Easy2.html#method.proxy_password)
    pub fn proxy_password(&mut self, pass: &str) -> Result<(), Error> {
        self.inner.proxy_password(pass)
    }

    /// Same as [`Easy2::proxy_auth`](struct.Easy2.html#method.proxy_auth)
    pub fn proxy_auth(&mut self, auth: &Auth) -> Result<(), Error> {
        self.inner.proxy_auth(auth)
    }

    /// Same as [`Easy2::netrc`](struct.Easy2.html#method.netrc)
    pub fn netrc(&mut self, netrc: NetRc) -> Result<(), Error> {
        self.inner.netrc(netrc)
    }

    // =========================================================================
    // HTTP Options

    /// Same as [`Easy2::autoreferer`](struct.Easy2.html#method.autoreferer)
    pub fn autoreferer(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.autoreferer(enable)
    }

    /// Same as [`Easy2::accept_encoding`](struct.Easy2.html#method.accept_encoding)
    pub fn accept_encoding(&mut self, encoding: &str) -> Result<(), Error> {
        self.inner.accept_encoding(encoding)
    }

    /// Same as [`Easy2::transfer_encoding`](struct.Easy2.html#method.transfer_encoding)
    pub fn transfer_encoding(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.transfer_encoding(enable)
    }

    /// Same as [`Easy2::follow_location`](struct.Easy2.html#method.follow_location)
    pub fn follow_location(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.follow_location(enable)
    }

    /// Same as [`Easy2::unrestricted_auth`](struct.Easy2.html#method.unrestricted_auth)
    pub fn unrestricted_auth(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.unrestricted_auth(enable)
    }

    /// Same as [`Easy2::max_redirections`](struct.Easy2.html#method.max_redirections)
    pub fn max_redirections(&mut self, max: u32) -> Result<(), Error> {
        self.inner.max_redirections(max)
    }

    /// Same as [`Easy2::put`](struct.Easy2.html#method.put)
    pub fn put(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.put(enable)
    }

    /// Same as [`Easy2::post`](struct.Easy2.html#method.post)
    pub fn post(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.post(enable)
    }

    /// Same as [`Easy2::post_field_copy`](struct.Easy2.html#method.post_field_copy)
    pub fn post_fields_copy(&mut self, data: &[u8]) -> Result<(), Error> {
        self.inner.post_fields_copy(data)
    }

    /// Same as [`Easy2::post_field_size`](struct.Easy2.html#method.post_field_size)
    pub fn post_field_size(&mut self, size: u64) -> Result<(), Error> {
        self.inner.post_field_size(size)
    }

    /// Same as [`Easy2::httppost`](struct.Easy2.html#method.httppost)
    pub fn httppost(&mut self, form: Form) -> Result<(), Error> {
        self.inner.httppost(form)
    }

    /// Same as [`Easy2::referer`](struct.Easy2.html#method.referer)
    pub fn referer(&mut self, referer: &str) -> Result<(), Error> {
        self.inner.referer(referer)
    }

    /// Same as [`Easy2::useragent`](struct.Easy2.html#method.useragent)
    pub fn useragent(&mut self, useragent: &str) -> Result<(), Error> {
        self.inner.useragent(useragent)
    }

    /// Same as [`Easy2::http_headers`](struct.Easy2.html#method.http_headers)
    pub fn http_headers(&mut self, list: List) -> Result<(), Error> {
        self.inner.http_headers(list)
    }

    /// Same as [`Easy2::cookie`](struct.Easy2.html#method.cookie)
    pub fn cookie(&mut self, cookie: &str) -> Result<(), Error> {
        self.inner.cookie(cookie)
    }

    /// Same as [`Easy2::cookie_file`](struct.Easy2.html#method.cookie_file)
    pub fn cookie_file<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Error> {
        self.inner.cookie_file(file)
    }

    /// Same as [`Easy2::cookie_jar`](struct.Easy2.html#method.cookie_jar)
    pub fn cookie_jar<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Error> {
        self.inner.cookie_jar(file)
    }

    /// Same as [`Easy2::cookie_session`](struct.Easy2.html#method.cookie_session)
    pub fn cookie_session(&mut self, session: bool) -> Result<(), Error> {
        self.inner.cookie_session(session)
    }

    /// Same as [`Easy2::cookie_list`](struct.Easy2.html#method.cookie_list)
    pub fn cookie_list(&mut self, cookie: &str) -> Result<(), Error> {
        self.inner.cookie_list(cookie)
    }

    /// Same as [`Easy2::get`](struct.Easy2.html#method.get)
    pub fn get(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.get(enable)
    }

    /// Same as [`Easy2::ignore_content_length`](struct.Easy2.html#method.ignore_content_length)
    pub fn ignore_content_length(&mut self, ignore: bool) -> Result<(), Error> {
        self.inner.ignore_content_length(ignore)
    }

    /// Same as [`Easy2::http_content_decoding`](struct.Easy2.html#method.http_content_decoding)
    pub fn http_content_decoding(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.http_content_decoding(enable)
    }

    /// Same as [`Easy2::http_transfer_decoding`](struct.Easy2.html#method.http_transfer_decoding)
    pub fn http_transfer_decoding(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.http_transfer_decoding(enable)
    }

    // =========================================================================
    // Protocol Options

    /// Same as [`Easy2::range`](struct.Easy2.html#method.range)
    pub fn range(&mut self, range: &str) -> Result<(), Error> {
        self.inner.range(range)
    }

    /// Same as [`Easy2::resume_from`](struct.Easy2.html#method.resume_from)
    pub fn resume_from(&mut self, from: u64) -> Result<(), Error> {
        self.inner.resume_from(from)
    }

    /// Same as [`Easy2::custom_request`](struct.Easy2.html#method.custom_request)
    pub fn custom_request(&mut self, request: &str) -> Result<(), Error> {
        self.inner.custom_request(request)
    }

    /// Same as [`Easy2::fetch_filetime`](struct.Easy2.html#method.fetch_filetime)
    pub fn fetch_filetime(&mut self, fetch: bool) -> Result<(), Error> {
        self.inner.fetch_filetime(fetch)
    }

    /// Same as [`Easy2::nobody`](struct.Easy2.html#method.nobody)
    pub fn nobody(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.nobody(enable)
    }

    /// Same as [`Easy2::in_filesize`](struct.Easy2.html#method.in_filesize)
    pub fn in_filesize(&mut self, size: u64) -> Result<(), Error> {
        self.inner.in_filesize(size)
    }

    /// Same as [`Easy2::upload`](struct.Easy2.html#method.upload)
    pub fn upload(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.upload(enable)
    }

    /// Same as [`Easy2::max_filesize`](struct.Easy2.html#method.max_filesize)
    pub fn max_filesize(&mut self, size: u64) -> Result<(), Error> {
        self.inner.max_filesize(size)
    }

    /// Same as [`Easy2::time_condition`](struct.Easy2.html#method.time_condition)
    pub fn time_condition(&mut self, cond: TimeCondition) -> Result<(), Error> {
        self.inner.time_condition(cond)
    }

    /// Same as [`Easy2::time_value`](struct.Easy2.html#method.time_value)
    pub fn time_value(&mut self, val: i64) -> Result<(), Error> {
        self.inner.time_value(val)
    }

    // =========================================================================
    // Connection Options

    /// Same as [`Easy2::timeout`](struct.Easy2.html#method.timeout)
    pub fn timeout(&mut self, timeout: Duration) -> Result<(), Error> {
        self.inner.timeout(timeout)
    }

    /// Same as [`Easy2::low_speed_limit`](struct.Easy2.html#method.low_speed_limit)
    pub fn low_speed_limit(&mut self, limit: u32) -> Result<(), Error> {
        self.inner.low_speed_limit(limit)
    }

    /// Same as [`Easy2::low_speed_time`](struct.Easy2.html#method.low_speed_time)
    pub fn low_speed_time(&mut self, dur: Duration) -> Result<(), Error> {
        self.inner.low_speed_time(dur)
    }

    /// Same as [`Easy2::max_send_speed`](struct.Easy2.html#method.max_send_speed)
    pub fn max_send_speed(&mut self, speed: u64) -> Result<(), Error> {
        self.inner.max_send_speed(speed)
    }

    /// Same as [`Easy2::max_recv_speed`](struct.Easy2.html#method.max_recv_speed)
    pub fn max_recv_speed(&mut self, speed: u64) -> Result<(), Error> {
        self.inner.max_recv_speed(speed)
    }

    /// Same as [`Easy2::max_connects`](struct.Easy2.html#method.max_connects)
    pub fn max_connects(&mut self, max: u32) -> Result<(), Error> {
        self.inner.max_connects(max)
    }

    /// Same as [`Easy2::fresh_connect`](struct.Easy2.html#method.fresh_connect)
    pub fn fresh_connect(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.fresh_connect(enable)
    }

    /// Same as [`Easy2::forbid_reuse`](struct.Easy2.html#method.forbid_reuse)
    pub fn forbid_reuse(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.forbid_reuse(enable)
    }

    /// Same as [`Easy2::connect_timeout`](struct.Easy2.html#method.connect_timeout)
    pub fn connect_timeout(&mut self, timeout: Duration) -> Result<(), Error> {
        self.inner.connect_timeout(timeout)
    }

    /// Same as [`Easy2::ip_resolve`](struct.Easy2.html#method.ip_resolve)
    pub fn ip_resolve(&mut self, resolve: IpResolve) -> Result<(), Error> {
        self.inner.ip_resolve(resolve)
    }

    /// Same as [`Easy2::resolve`](struct.Easy2.html#method.resolve)
    pub fn resolve(&mut self, list: List) -> Result<(), Error> {
        self.inner.resolve(list)
    }

    /// Same as [`Easy2::connect_only`](struct.Easy2.html#method.connect_only)
    pub fn connect_only(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.connect_only(enable)
    }

    // =========================================================================
    // SSL/Security Options

    /// Same as [`Easy2::ssl_cert`](struct.Easy2.html#method.ssl_cert)
    pub fn ssl_cert<P: AsRef<Path>>(&mut self, cert: P) -> Result<(), Error> {
        self.inner.ssl_cert(cert)
    }

    /// Same as [`Easy2::ssl_cert_type`](struct.Easy2.html#method.ssl_cert_type)
    pub fn ssl_cert_type(&mut self, kind: &str) -> Result<(), Error> {
        self.inner.ssl_cert_type(kind)
    }

    /// Same as [`Easy2::ssl_key`](struct.Easy2.html#method.ssl_key)
    pub fn ssl_key<P: AsRef<Path>>(&mut self, key: P) -> Result<(), Error> {
        self.inner.ssl_key(key)
    }

    /// Same as [`Easy2::ssl_key_type`](struct.Easy2.html#method.ssl_key_type)
    pub fn ssl_key_type(&mut self, kind: &str) -> Result<(), Error> {
        self.inner.ssl_key_type(kind)
    }

    /// Same as [`Easy2::key_password`](struct.Easy2.html#method.key_password)
    pub fn key_password(&mut self, password: &str) -> Result<(), Error> {
        self.inner.key_password(password)
    }

    /// Same as [`Easy2::ssl_engine`](struct.Easy2.html#method.ssl_engine)
    pub fn ssl_engine(&mut self, engine: &str) -> Result<(), Error> {
        self.inner.ssl_engine(engine)
    }

    /// Same as [`Easy2::ssl_engine_default`](struct.Easy2.html#method.ssl_engine_default)
    pub fn ssl_engine_default(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.ssl_engine_default(enable)
    }

    /// Same as [`Easy2::http_version`](struct.Easy2.html#method.http_version)
    pub fn http_version(&mut self, version: HttpVersion) -> Result<(), Error> {
        self.inner.http_version(version)
    }

    /// Same as [`Easy2::ssl_version`](struct.Easy2.html#method.ssl_version)
    pub fn ssl_version(&mut self, version: SslVersion) -> Result<(), Error> {
        self.inner.ssl_version(version)
    }

    /// Same as [`Easy2::ssl_min_max_version`](struct.Easy2.html#method.ssl_min_max_version)
    pub fn ssl_min_max_version(
        &mut self,
        min_version: SslVersion,
        max_version: SslVersion,
    ) -> Result<(), Error> {
        self.inner.ssl_min_max_version(min_version, max_version)
    }

    /// Same as [`Easy2::ssl_verify_host`](struct.Easy2.html#method.ssl_verify_host)
    pub fn ssl_verify_host(&mut self, verify: bool) -> Result<(), Error> {
        self.inner.ssl_verify_host(verify)
    }

    /// Same as [`Easy2::ssl_verify_peer`](struct.Easy2.html#method.ssl_verify_peer)
    pub fn ssl_verify_peer(&mut self, verify: bool) -> Result<(), Error> {
        self.inner.ssl_verify_peer(verify)
    }

    /// Same as [`Easy2::cainfo`](struct.Easy2.html#method.cainfo)
    pub fn cainfo<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.inner.cainfo(path)
    }

    /// Same as [`Easy2::issuer_cert`](struct.Easy2.html#method.issuer_cert)
    pub fn issuer_cert<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.inner.issuer_cert(path)
    }

    /// Same as [`Easy2::capath`](struct.Easy2.html#method.capath)
    pub fn capath<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.inner.capath(path)
    }

    /// Same as [`Easy2::crlfile`](struct.Easy2.html#method.crlfile)
    pub fn crlfile<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.inner.crlfile(path)
    }

    /// Same as [`Easy2::certinfo`](struct.Easy2.html#method.certinfo)
    pub fn certinfo(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.certinfo(enable)
    }

    /// Same as [`Easy2::random_file`](struct.Easy2.html#method.random_file)
    pub fn random_file<P: AsRef<Path>>(&mut self, p: P) -> Result<(), Error> {
        self.inner.random_file(p)
    }

    /// Same as [`Easy2::egd_socket`](struct.Easy2.html#method.egd_socket)
    pub fn egd_socket<P: AsRef<Path>>(&mut self, p: P) -> Result<(), Error> {
        self.inner.egd_socket(p)
    }

    /// Same as [`Easy2::ssl_cipher_list`](struct.Easy2.html#method.ssl_cipher_list)
    pub fn ssl_cipher_list(&mut self, ciphers: &str) -> Result<(), Error> {
        self.inner.ssl_cipher_list(ciphers)
    }

    /// Same as [`Easy2::ssl_sessionid_cache`](struct.Easy2.html#method.ssl_sessionid_cache)
    pub fn ssl_sessionid_cache(&mut self, enable: bool) -> Result<(), Error> {
        self.inner.ssl_sessionid_cache(enable)
    }

    /// Same as [`Easy2::ssl_options`](struct.Easy2.html#method.ssl_options)
    pub fn ssl_options(&mut self, bits: &SslOpt) -> Result<(), Error> {
        self.inner.ssl_options(bits)
    }

    // =========================================================================
    // getters

    /// Same as [`Easy2::time_condition_unmet`](struct.Easy2.html#method.time_condition_unmet)
    pub fn time_condition_unmet(&mut self) -> Result<bool, Error> {
        self.inner.time_condition_unmet()
    }

    /// Same as [`Easy2::effective_url`](struct.Easy2.html#method.effective_url)
    pub fn effective_url(&mut self) -> Result<Option<&str>, Error> {
        self.inner.effective_url()
    }

    /// Same as [`Easy2::effective_url_bytes`](struct.Easy2.html#method.effective_url_bytes)
    pub fn effective_url_bytes(&mut self) -> Result<Option<&[u8]>, Error> {
        self.inner.effective_url_bytes()
    }

    /// Same as [`Easy2::response_code`](struct.Easy2.html#method.response_code)
    pub fn response_code(&mut self) -> Result<u32, Error> {
        self.inner.response_code()
    }

    /// Same as [`Easy2::http_connectcode`](struct.Easy2.html#method.http_connectcode)
    pub fn http_connectcode(&mut self) -> Result<u32, Error> {
        self.inner.http_connectcode()
    }

    /// Same as [`Easy2::filetime`](struct.Easy2.html#method.filetime)
    pub fn filetime(&mut self) -> Result<Option<i64>, Error> {
        self.inner.filetime()
    }

    /// Same as [`Easy2::download_size`](struct.Easy2.html#method.download_size)
    pub fn download_size(&mut self) -> Result<f64, Error> {
        self.inner.download_size()
    }
    /// Same as [`Easy2::content_length_download`](struct.Easy2.html#method.content_length_download)
    pub fn content_length_download(&mut self) -> Result<f64, Error> {
        self.inner.content_length_download()
    }

    /// Same as [`Easy2::total_time`](struct.Easy2.html#method.total_time)
    pub fn total_time(&mut self) -> Result<Duration, Error> {
        self.inner.total_time()
    }

    /// Same as [`Easy2::namelookup_time`](struct.Easy2.html#method.namelookup_time)
    pub fn namelookup_time(&mut self) -> Result<Duration, Error> {
        self.inner.namelookup_time()
    }

    /// Same as [`Easy2::connect_time`](struct.Easy2.html#method.connect_time)
    pub fn connect_time(&mut self) -> Result<Duration, Error> {
        self.inner.connect_time()
    }

    /// Same as [`Easy2::appconnect_time`](struct.Easy2.html#method.appconnect_time)
    pub fn appconnect_time(&mut self) -> Result<Duration, Error> {
        self.inner.appconnect_time()
    }

    /// Same as [`Easy2::pretransfer_time`](struct.Easy2.html#method.pretransfer_time)
    pub fn pretransfer_time(&mut self) -> Result<Duration, Error> {
        self.inner.pretransfer_time()
    }

    /// Same as [`Easy2::starttransfer_time`](struct.Easy2.html#method.starttransfer_time)
    pub fn starttransfer_time(&mut self) -> Result<Duration, Error> {
        self.inner.starttransfer_time()
    }

    /// Same as [`Easy2::redirect_time`](struct.Easy2.html#method.redirect_time)
    pub fn redirect_time(&mut self) -> Result<Duration, Error> {
        self.inner.redirect_time()
    }

    /// Same as [`Easy2::redirect_count`](struct.Easy2.html#method.redirect_count)
    pub fn redirect_count(&mut self) -> Result<u32, Error> {
        self.inner.redirect_count()
    }

    /// Same as [`Easy2::redirect_url`](struct.Easy2.html#method.redirect_url)
    pub fn redirect_url(&mut self) -> Result<Option<&str>, Error> {
        self.inner.redirect_url()
    }

    /// Same as [`Easy2::redirect_url_bytes`](struct.Easy2.html#method.redirect_url_bytes)
    pub fn redirect_url_bytes(&mut self) -> Result<Option<&[u8]>, Error> {
        self.inner.redirect_url_bytes()
    }

    /// Same as [`Easy2::header_size`](struct.Easy2.html#method.header_size)
    pub fn header_size(&mut self) -> Result<u64, Error> {
        self.inner.header_size()
    }

    /// Same as [`Easy2::request_size`](struct.Easy2.html#method.request_size)
    pub fn request_size(&mut self) -> Result<u64, Error> {
        self.inner.request_size()
    }

    /// Same as [`Easy2::content_type`](struct.Easy2.html#method.content_type)
    pub fn content_type(&mut self) -> Result<Option<&str>, Error> {
        self.inner.content_type()
    }

    /// Same as [`Easy2::content_type_bytes`](struct.Easy2.html#method.content_type_bytes)
    pub fn content_type_bytes(&mut self) -> Result<Option<&[u8]>, Error> {
        self.inner.content_type_bytes()
    }

    /// Same as [`Easy2::os_errno`](struct.Easy2.html#method.os_errno)
    pub fn os_errno(&mut self) -> Result<i32, Error> {
        self.inner.os_errno()
    }

    /// Same as [`Easy2::primary_ip`](struct.Easy2.html#method.primary_ip)
    pub fn primary_ip(&mut self) -> Result<Option<&str>, Error> {
        self.inner.primary_ip()
    }

    /// Same as [`Easy2::primary_port`](struct.Easy2.html#method.primary_port)
    pub fn primary_port(&mut self) -> Result<u16, Error> {
        self.inner.primary_port()
    }

    /// Same as [`Easy2::local_ip`](struct.Easy2.html#method.local_ip)
    pub fn local_ip(&mut self) -> Result<Option<&str>, Error> {
        self.inner.local_ip()
    }

    /// Same as [`Easy2::local_port`](struct.Easy2.html#method.local_port)
    pub fn local_port(&mut self) -> Result<u16, Error> {
        self.inner.local_port()
    }

    /// Same as [`Easy2::cookies`](struct.Easy2.html#method.cookies)
    pub fn cookies(&mut self) -> Result<List, Error> {
        self.inner.cookies()
    }

    /// Same as [`Easy2::pipewait`](struct.Easy2.html#method.pipewait)
    pub fn pipewait(&mut self, wait: bool) -> Result<(), Error> {
        self.inner.pipewait(wait)
    }

    // =========================================================================
    // Other methods

    /// Same as [`Easy2::perform`](struct.Easy2.html#method.perform)
    pub fn perform(&self) -> Result<(), Error> {
        assert!(self.inner.get_ref().borrowed.get().is_null());
        self.do_perform()
    }

    fn do_perform(&self) -> Result<(), Error> {
        // We don't allow recursive invocations of `perform` because we're
        // invoking `FnMut`closures behind a `&self` pointer. This flag acts as
        // our own `RefCell` borrow flag sorta.
        if self.inner.get_ref().running.get() {
            return Err(Error::new(curl_sys::CURLE_FAILED_INIT));
        }

        self.inner.get_ref().running.set(true);
        struct Reset<'a>(&'a Cell<bool>);
        impl<'a> Drop for Reset<'a> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _reset = Reset(&self.inner.get_ref().running);

        self.inner.perform()
    }

    /// Creates a new scoped transfer which can be used to set callbacks and
    /// data which only live for the scope of the returned object.
    ///
    /// An `Easy` handle is often reused between different requests to cache
    /// connections to servers, but often the lifetime of the data as part of
    /// each transfer is unique. This function serves as an ability to share an
    /// `Easy` across many transfers while ergonomically using possibly
    /// stack-local data as part of each transfer.
    ///
    /// Configuration can be set on the `Easy` and then a `Transfer` can be
    /// created to set scoped configuration (like callbacks). Finally, the
    /// `perform` method on the `Transfer` function can be used.
    ///
    /// When the `Transfer` option is dropped then all configuration set on the
    /// transfer itself will be reset.
    pub fn transfer<'data, 'easy>(&'easy mut self) -> Transfer<'easy, 'data> {
        assert!(!self.inner.get_ref().running.get());
        Transfer {
            data: Box::new(Callbacks::default()),
            easy: self,
        }
    }

    /// Same as [`Easy2::unpause_read`](struct.Easy2.html#method.unpause_read)
    pub fn unpause_read(&self) -> Result<(), Error> {
        self.inner.unpause_read()
    }

    /// Same as [`Easy2::unpause_write`](struct.Easy2.html#method.unpause_write)
    pub fn unpause_write(&self) -> Result<(), Error> {
        self.inner.unpause_write()
    }

    /// Same as [`Easy2::url_encode`](struct.Easy2.html#method.url_encode)
    pub fn url_encode(&mut self, s: &[u8]) -> String {
        self.inner.url_encode(s)
    }

    /// Same as [`Easy2::url_decode`](struct.Easy2.html#method.url_decode)
    pub fn url_decode(&mut self, s: &str) -> Vec<u8> {
        self.inner.url_decode(s)
    }

    /// Same as [`Easy2::reset`](struct.Easy2.html#method.reset)
    pub fn reset(&mut self) {
        self.inner.reset()
    }

    /// Same as [`Easy2::recv`](struct.Easy2.html#method.recv)
    pub fn recv(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.inner.recv(data)
    }

    /// Same as [`Easy2::send`](struct.Easy2.html#method.send)
    pub fn send(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.inner.send(data)
    }

    /// Same as [`Easy2::raw`](struct.Easy2.html#method.raw)
    pub fn raw(&self) -> *mut curl_sys::CURL {
        self.inner.raw()
    }

    /// Same as [`Easy2::take_error_buf`](struct.Easy2.html#method.take_error_buf)
    pub fn take_error_buf(&self) -> Option<String> {
        self.inner.take_error_buf()
    }
}

impl EasyData {
    /// An unsafe function to get the appropriate callback field.
    ///
    /// We can have callbacks configured from one of two different sources.
    /// We could either have a callback from the `borrowed` field, callbacks on
    /// an ephemeral `Transfer`, or the `owned` field which are `'static`
    /// callbacks that live for the lifetime of this `EasyData`.
    ///
    /// The first set of callbacks are unsafe to access because they're actually
    /// owned elsewhere and we're just aliasing. Additionally they don't
    /// technically live long enough for us to access them, so they're hidden
    /// behind unsafe pointers and casts.
    ///
    /// This function returns `&'a mut T` but that's actually somewhat of a lie.
    /// The value should **not be stored to** nor should it be used for the full
    /// lifetime of `'a`, but rather immediately in the local scope.
    ///
    /// Basically this is just intended to acquire a callback, invoke it, and
    /// then stop. Nothing else. Super unsafe.
    unsafe fn callback<'a, T, F>(&'a mut self, f: F) -> Option<&'a mut T>
    where
        F: for<'b> Fn(&'b mut Callbacks<'static>) -> &'b mut Option<T>,
    {
        let ptr = self.borrowed.get();
        if !ptr.is_null() {
            let val = f(&mut *ptr);
            if val.is_some() {
                return val.as_mut();
            }
        }
        f(&mut self.owned).as_mut()
    }
}

impl Handler for EasyData {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        unsafe {
            match self.callback(|s| &mut s.write) {
                Some(write) => write(data),
                None => Ok(data.len()),
            }
        }
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        unsafe {
            match self.callback(|s| &mut s.read) {
                Some(read) => read(data),
                None => Ok(0),
            }
        }
    }

    fn seek(&mut self, whence: SeekFrom) -> SeekResult {
        unsafe {
            match self.callback(|s| &mut s.seek) {
                Some(seek) => seek(whence),
                None => SeekResult::CantSeek,
            }
        }
    }

    fn debug(&mut self, kind: InfoType, data: &[u8]) {
        unsafe {
            match self.callback(|s| &mut s.debug) {
                Some(debug) => debug(kind, data),
                None => handler::debug(kind, data),
            }
        }
    }

    fn header(&mut self, data: &[u8]) -> bool {
        unsafe {
            match self.callback(|s| &mut s.header) {
                Some(header) => header(data),
                None => true,
            }
        }
    }

    fn progress(&mut self, dltotal: f64, dlnow: f64, ultotal: f64, ulnow: f64) -> bool {
        unsafe {
            match self.callback(|s| &mut s.progress) {
                Some(progress) => progress(dltotal, dlnow, ultotal, ulnow),
                None => true,
            }
        }
    }

    fn ssl_ctx(&mut self, cx: *mut c_void) -> Result<(), Error> {
        unsafe {
            match self.callback(|s| &mut s.ssl_ctx) {
                Some(ssl_ctx) => ssl_ctx(cx),
                None => handler::ssl_ctx(cx),
            }
        }
    }
}

impl fmt::Debug for EasyData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "callbacks ...".fmt(f)
    }
}

impl<'easy, 'data> Transfer<'easy, 'data> {
    /// Same as `Easy::write_function`, just takes a non `'static` lifetime
    /// corresponding to the lifetime of this transfer.
    pub fn write_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> Result<usize, WriteError> + 'data,
    {
        self.data.write = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::read_function`, just takes a non `'static` lifetime
    /// corresponding to the lifetime of this transfer.
    pub fn read_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(&mut [u8]) -> Result<usize, ReadError> + 'data,
    {
        self.data.read = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::seek_function`, just takes a non `'static` lifetime
    /// corresponding to the lifetime of this transfer.
    pub fn seek_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(SeekFrom) -> SeekResult + 'data,
    {
        self.data.seek = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::progress_function`, just takes a non `'static` lifetime
    /// corresponding to the lifetime of this transfer.
    pub fn progress_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(f64, f64, f64, f64) -> bool + 'data,
    {
        self.data.progress = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::ssl_ctx_function`, just takes a non `'static`
    /// lifetime corresponding to the lifetime of this transfer.
    pub fn ssl_ctx_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(*mut c_void) -> Result<(), Error> + Send + 'data,
    {
        self.data.ssl_ctx = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::debug_function`, just takes a non `'static` lifetime
    /// corresponding to the lifetime of this transfer.
    pub fn debug_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(InfoType, &[u8]) + 'data,
    {
        self.data.debug = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::header_function`, just takes a non `'static` lifetime
    /// corresponding to the lifetime of this transfer.
    pub fn header_function<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> bool + 'data,
    {
        self.data.header = Some(Box::new(f));
        Ok(())
    }

    /// Same as `Easy::perform`.
    pub fn perform(&self) -> Result<(), Error> {
        let inner = self.easy.inner.get_ref();

        // Note that we're casting a `&self` pointer to a `*mut`, and then
        // during the invocation of this call we're going to invoke `FnMut`
        // closures that we ourselves own.
        //
        // This should be ok, however, because `do_perform` checks for recursive
        // invocations of `perform` and disallows them. Our type also isn't
        // `Sync`.
        inner.borrowed.set(&*self.data as *const _ as *mut _);

        // Make sure to reset everything back to the way it was before when
        // we're done.
        struct Reset<'a>(&'a Cell<*mut Callbacks<'static>>);
        impl<'a> Drop for Reset<'a> {
            fn drop(&mut self) {
                self.0.set(ptr::null_mut());
            }
        }
        let _reset = Reset(&inner.borrowed);

        self.easy.do_perform()
    }

    /// Same as `Easy::unpause_read`.
    pub fn unpause_read(&self) -> Result<(), Error> {
        self.easy.unpause_read()
    }

    /// Same as `Easy::unpause_write`
    pub fn unpause_write(&self) -> Result<(), Error> {
        self.easy.unpause_write()
    }
}

impl<'easy, 'data> fmt::Debug for Transfer<'easy, 'data> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Transfer")
            .field("easy", &self.easy)
            .finish()
    }
}

impl<'easy, 'data> Drop for Transfer<'easy, 'data> {
    fn drop(&mut self) {
        // Extra double check to make sure we don't leak a pointer to ourselves.
        assert!(self.easy.inner.get_ref().borrowed.get().is_null());
    }
}
