//! Multi - initiating multiple requests simultaneously

use std::fmt;
use std::marker;
use std::time::Duration;

use curl_sys;
use libc::{c_char, c_int, c_long, c_short, c_void};

#[cfg(unix)]
use libc::{pollfd, POLLIN, POLLOUT, POLLPRI};

use easy::{Easy, Easy2, List};
use panic;
use {Error, MultiError};

/// A multi handle for initiating multiple connections simultaneously.
///
/// This structure corresponds to `CURLM` in libcurl and provides the ability to
/// have multiple transfers in flight simultaneously. This handle is then used
/// to manage each transfer. The main purpose of a `CURLM` is for the
/// *application* to drive the I/O rather than libcurl itself doing all the
/// blocking. Methods like `action` allow the application to inform libcurl of
/// when events have happened.
///
/// Lots more documentation can be found on the libcurl [multi tutorial] where
/// the APIs correspond pretty closely with this crate.
///
/// [multi tutorial]: https://curl.haxx.se/libcurl/c/libcurl-multi.html
pub struct Multi {
    raw: *mut curl_sys::CURLM,
    data: Box<MultiData>,
}

struct MultiData {
    socket: Box<dyn FnMut(Socket, SocketEvents, usize) + Send>,
    timer: Box<dyn FnMut(Option<Duration>) -> bool + Send>,
}

/// Message from the `messages` function of a multi handle.
///
/// Currently only indicates whether a transfer is done.
pub struct Message<'multi> {
    ptr: *mut curl_sys::CURLMsg,
    _multi: &'multi Multi,
}

/// Wrapper around an easy handle while it's owned by a multi handle.
///
/// Once an easy handle has been added to a multi handle then it can no longer
/// be used via `perform`. This handle is also used to remove the easy handle
/// from the multi handle when desired.
pub struct EasyHandle {
    easy: Easy,
    // This is now effectively bound to a `Multi`, so it is no longer sendable.
    _marker: marker::PhantomData<&'static Multi>,
}

/// Wrapper around an easy handle while it's owned by a multi handle.
///
/// Once an easy handle has been added to a multi handle then it can no longer
/// be used via `perform`. This handle is also used to remove the easy handle
/// from the multi handle when desired.
pub struct Easy2Handle<H> {
    easy: Easy2<H>,
    // This is now effectively bound to a `Multi`, so it is no longer sendable.
    _marker: marker::PhantomData<&'static Multi>,
}

/// Notification of the events that have happened on a socket.
///
/// This type is passed as an argument to the `action` method on a multi handle
/// to indicate what events have occurred on a socket.
pub struct Events {
    bits: c_int,
}

/// Notification of events that are requested on a socket.
///
/// This type is yielded to the `socket_function` callback to indicate what
/// events are requested on a socket.
pub struct SocketEvents {
    bits: c_int,
}

/// Raw underlying socket type that the multi handles use
pub type Socket = curl_sys::curl_socket_t;

/// File descriptor to wait on for use with the `wait` method on a multi handle.
pub struct WaitFd {
    inner: curl_sys::curl_waitfd,
}

impl Multi {
    /// Creates a new multi session through which multiple HTTP transfers can be
    /// initiated.
    pub fn new() -> Multi {
        unsafe {
            ::init();
            let ptr = curl_sys::curl_multi_init();
            assert!(!ptr.is_null());
            Multi {
                raw: ptr,
                data: Box::new(MultiData {
                    socket: Box::new(|_, _, _| ()),
                    timer: Box::new(|_| true),
                }),
            }
        }
    }

    /// Set the callback informed about what to wait for
    ///
    /// When the `action` function runs, it informs the application about
    /// updates in the socket (file descriptor) status by doing none, one, or
    /// multiple calls to the socket callback. The callback gets status updates
    /// with changes since the previous time the callback was called. See
    /// `action` for more details on how the callback is used and should work.
    ///
    /// The `SocketEvents` parameter informs the callback on the status of the
    /// given socket, and the methods on that type can be used to learn about
    /// what's going on with the socket.
    ///
    /// The third `usize` parameter is a custom value set by the `assign` method
    /// below.
    pub fn socket_function<F>(&mut self, f: F) -> Result<(), MultiError>
    where
        F: FnMut(Socket, SocketEvents, usize) + Send + 'static,
    {
        self._socket_function(Box::new(f))
    }

    fn _socket_function(
        &mut self,
        f: Box<dyn FnMut(Socket, SocketEvents, usize) + Send>,
    ) -> Result<(), MultiError> {
        self.data.socket = f;
        let cb: curl_sys::curl_socket_callback = cb;
        self.setopt_ptr(
            curl_sys::CURLMOPT_SOCKETFUNCTION,
            cb as usize as *const c_char,
        )?;
        let ptr = &*self.data as *const _;
        self.setopt_ptr(curl_sys::CURLMOPT_SOCKETDATA, ptr as *const c_char)?;
        return Ok(());

        // TODO: figure out how to expose `_easy`
        extern "C" fn cb(
            _easy: *mut curl_sys::CURL,
            socket: curl_sys::curl_socket_t,
            what: c_int,
            userptr: *mut c_void,
            socketp: *mut c_void,
        ) -> c_int {
            panic::catch(|| unsafe {
                let f = &mut (*(userptr as *mut MultiData)).socket;
                f(socket, SocketEvents { bits: what }, socketp as usize)
            });
            0
        }
    }

    /// Set data to associate with an internal socket
    ///
    /// This function creates an association in the multi handle between the
    /// given socket and a private token of the application. This is designed
    /// for `action` uses.
    ///
    /// When set, the token will be passed to all future socket callbacks for
    /// the specified socket.
    ///
    /// If the given socket isn't already in use by libcurl, this function will
    /// return an error.
    ///
    /// libcurl only keeps one single token associated with a socket, so
    /// calling this function several times for the same socket will make the
    /// last set token get used.
    ///
    /// The idea here being that this association (socket to token) is something
    /// that just about every application that uses this API will need and then
    /// libcurl can just as well do it since it already has an internal hash
    /// table lookup for this.
    ///
    /// # Typical Usage
    ///
    /// In a typical application you allocate a struct or at least use some kind
    /// of semi-dynamic data for each socket that we must wait for action on
    /// when using the `action` approach.
    ///
    /// When our socket-callback gets called by libcurl and we get to know about
    /// yet another socket to wait for, we can use `assign` to point out the
    /// particular data so that when we get updates about this same socket
    /// again, we don't have to find the struct associated with this socket by
    /// ourselves.
    pub fn assign(&self, socket: Socket, token: usize) -> Result<(), MultiError> {
        unsafe {
            cvt(curl_sys::curl_multi_assign(
                self.raw,
                socket,
                token as *mut _,
            ))?;
            Ok(())
        }
    }

    /// Set callback to receive timeout values
    ///
    /// Certain features, such as timeouts and retries, require you to call
    /// libcurl even when there is no activity on the file descriptors.
    ///
    /// Your callback function should install a non-repeating timer with the
    /// interval specified. Each time that timer fires, call either `action` or
    /// `perform`, depending on which interface you use.
    ///
    /// A timeout value of `None` means you should delete your timer.
    ///
    /// A timeout value of 0 means you should call `action` or `perform` (once)
    /// as soon as possible.
    ///
    /// This callback will only be called when the timeout changes.
    ///
    /// The timer callback should return `true` on success, and `false` on
    /// error. This callback can be used instead of, or in addition to,
    /// `get_timeout`.
    pub fn timer_function<F>(&mut self, f: F) -> Result<(), MultiError>
    where
        F: FnMut(Option<Duration>) -> bool + Send + 'static,
    {
        self._timer_function(Box::new(f))
    }

    fn _timer_function(
        &mut self,
        f: Box<dyn FnMut(Option<Duration>) -> bool + Send>,
    ) -> Result<(), MultiError> {
        self.data.timer = f;
        let cb: curl_sys::curl_multi_timer_callback = cb;
        self.setopt_ptr(
            curl_sys::CURLMOPT_TIMERFUNCTION,
            cb as usize as *const c_char,
        )?;
        let ptr = &*self.data as *const _;
        self.setopt_ptr(curl_sys::CURLMOPT_TIMERDATA, ptr as *const c_char)?;
        return Ok(());

        // TODO: figure out how to expose `_multi`
        extern "C" fn cb(
            _multi: *mut curl_sys::CURLM,
            timeout_ms: c_long,
            user: *mut c_void,
        ) -> c_int {
            let keep_going = panic::catch(|| unsafe {
                let f = &mut (*(user as *mut MultiData)).timer;
                if timeout_ms == -1 {
                    f(None)
                } else {
                    f(Some(Duration::from_millis(timeout_ms as u64)))
                }
            })
            .unwrap_or(false);
            if keep_going {
                0
            } else {
                -1
            }
        }
    }

    /// Enable or disable HTTP pipelining and multiplexing.
    ///
    /// When http_1 is true, enable HTTP/1.1 pipelining, which means that if
    /// you add a second request that can use an already existing connection,
    /// the second request will be "piped" on the same connection rather than
    /// being executed in parallel.
    ///
    /// When multiplex is true, enable HTTP/2 multiplexing, which means that
    /// follow-up requests can re-use an existing connection and send the new
    /// request multiplexed over that at the same time as other transfers are
    /// already using that single connection.
    pub fn pipelining(&mut self, http_1: bool, multiplex: bool) -> Result<(), MultiError> {
        let bitmask = if http_1 { curl_sys::CURLPIPE_HTTP1 } else { 0 }
            | if multiplex {
                curl_sys::CURLPIPE_MULTIPLEX
            } else {
                0
            };
        self.setopt_long(curl_sys::CURLMOPT_PIPELINING, bitmask)
    }

    /// Sets the max number of connections to a single host.
    ///
    /// Pass a long to indicate the max number of simultaneously open connections
    /// to a single host (a host being the same as a host name + port number pair).
    /// For each new session to a host, libcurl will open up a new connection up to the
    /// limit set by the provided value. When the limit is reached, the sessions will
    /// be pending until a connection becomes available. If pipelining is enabled,
    /// libcurl will try to pipeline if the host is capable of it.
    pub fn set_max_host_connections(&mut self, val: usize) -> Result<(), MultiError> {
        self.setopt_long(curl_sys::CURLMOPT_MAX_HOST_CONNECTIONS, val as c_long)
    }

    /// Sets the max simultaneously open connections.
    ///
    /// The set number will be used as the maximum number of simultaneously open
    /// connections in total using this multi handle. For each new session,
    /// libcurl will open a new connection up to the limit set by the provided
    /// value. When the limit is reached, the sessions will be pending until
    /// there are available connections. If pipelining is enabled, libcurl will
    /// try to pipeline or use multiplexing if the host is capable of it.
    pub fn set_max_total_connections(&mut self, val: usize) -> Result<(), MultiError> {
        self.setopt_long(curl_sys::CURLMOPT_MAX_TOTAL_CONNECTIONS, val as c_long)
    }

    /// Set size of connection cache.
    ///
    /// The set number will be used as the maximum amount of simultaneously open
    /// connections that libcurl may keep in its connection cache after
    /// completed use. By default libcurl will enlarge the size for each added
    /// easy handle to make it fit 4 times the number of added easy handles.
    ///
    /// By setting this option, you can prevent the cache size from growing
    /// beyond the limit set by you.
    ///
    /// When the cache is full, curl closes the oldest one in the cache to
    /// prevent the number of open connections from increasing.
    ///
    /// See [`set_max_total_connections`](#method.set_max_total_connections) for
    /// limiting the number of active connections.
    pub fn set_max_connects(&mut self, val: usize) -> Result<(), MultiError> {
        self.setopt_long(curl_sys::CURLMOPT_MAXCONNECTS, val as c_long)
    }

    /// Sets the pipeline length.
    ///
    /// This sets the max number that will be used as the maximum amount of
    /// outstanding requests in an HTTP/1.1 pipelined connection. This option
    /// is only used for HTTP/1.1 pipelining, and not HTTP/2 multiplexing.
    pub fn set_pipeline_length(&mut self, val: usize) -> Result<(), MultiError> {
        self.setopt_long(curl_sys::CURLMOPT_MAX_PIPELINE_LENGTH, val as c_long)
    }

    fn setopt_long(&mut self, opt: curl_sys::CURLMoption, val: c_long) -> Result<(), MultiError> {
        unsafe { cvt(curl_sys::curl_multi_setopt(self.raw, opt, val)) }
    }

    fn setopt_ptr(
        &mut self,
        opt: curl_sys::CURLMoption,
        val: *const c_char,
    ) -> Result<(), MultiError> {
        unsafe { cvt(curl_sys::curl_multi_setopt(self.raw, opt, val)) }
    }

    /// Add an easy handle to a multi session
    ///
    /// Adds a standard easy handle to the multi stack. This function call will
    /// make this multi handle control the specified easy handle.
    ///
    /// When an easy interface is added to a multi handle, it will use a shared
    /// connection cache owned by the multi handle. Removing and adding new easy
    /// handles will not affect the pool of connections or the ability to do
    /// connection re-use.
    ///
    /// If you have `timer_function` set in the multi handle (and you really
    /// should if you're working event-based with `action` and friends), that
    /// callback will be called from within this function to ask for an updated
    /// timer so that your main event loop will get the activity on this handle
    /// to get started.
    ///
    /// The easy handle will remain added to the multi handle until you remove
    /// it again with `remove` on the returned handle - even when a transfer
    /// with that specific easy handle is completed.
    pub fn add(&self, mut easy: Easy) -> Result<EasyHandle, MultiError> {
        // Clear any configuration set by previous transfers because we're
        // moving this into a `Send+'static` situation now basically.
        easy.transfer();

        unsafe {
            cvt(curl_sys::curl_multi_add_handle(self.raw, easy.raw()))?;
        }
        Ok(EasyHandle {
            easy: easy,
            _marker: marker::PhantomData,
        })
    }

    /// Same as `add`, but works with the `Easy2` type.
    pub fn add2<H>(&self, easy: Easy2<H>) -> Result<Easy2Handle<H>, MultiError> {
        unsafe {
            cvt(curl_sys::curl_multi_add_handle(self.raw, easy.raw()))?;
        }
        Ok(Easy2Handle {
            easy: easy,
            _marker: marker::PhantomData,
        })
    }

    /// Remove an easy handle from this multi session
    ///
    /// Removes the easy handle from this multi handle. This will make the
    /// returned easy handle be removed from this multi handle's control.
    ///
    /// When the easy handle has been removed from a multi stack, it is again
    /// perfectly legal to invoke `perform` on it.
    ///
    /// Removing an easy handle while being used is perfectly legal and will
    /// effectively halt the transfer in progress involving that easy handle.
    /// All other easy handles and transfers will remain unaffected.
    pub fn remove(&self, easy: EasyHandle) -> Result<Easy, MultiError> {
        unsafe {
            cvt(curl_sys::curl_multi_remove_handle(
                self.raw,
                easy.easy.raw(),
            ))?;
        }
        Ok(easy.easy)
    }

    /// Same as `remove`, but for `Easy2Handle`.
    pub fn remove2<H>(&self, easy: Easy2Handle<H>) -> Result<Easy2<H>, MultiError> {
        unsafe {
            cvt(curl_sys::curl_multi_remove_handle(
                self.raw,
                easy.easy.raw(),
            ))?;
        }
        Ok(easy.easy)
    }

    /// Read multi stack informationals
    ///
    /// Ask the multi handle if there are any messages/informationals from the
    /// individual transfers. Messages may include informationals such as an
    /// error code from the transfer or just the fact that a transfer is
    /// completed. More details on these should be written down as well.
    pub fn messages<F>(&self, mut f: F)
    where
        F: FnMut(Message),
    {
        self._messages(&mut f)
    }

    fn _messages(&self, f: &mut dyn FnMut(Message)) {
        let mut queue = 0;
        unsafe {
            loop {
                let ptr = curl_sys::curl_multi_info_read(self.raw, &mut queue);
                if ptr.is_null() {
                    break;
                }
                f(Message {
                    ptr: ptr,
                    _multi: self,
                })
            }
        }
    }

    /// Inform of reads/writes available data given an action
    ///
    /// When the application has detected action on a socket handled by libcurl,
    /// it should call this function with the sockfd argument set to
    /// the socket with the action. When the events on a socket are known, they
    /// can be passed `events`. When the events on a socket are unknown, pass
    /// `Events::new()` instead, and libcurl will test the descriptor
    /// internally.
    ///
    /// The returned integer will contain the number of running easy handles
    /// within the multi handle. When this number reaches zero, all transfers
    /// are complete/done. When you call `action` on a specific socket and the
    /// counter decreases by one, it DOES NOT necessarily mean that this exact
    /// socket/transfer is the one that completed. Use `messages` to figure out
    /// which easy handle that completed.
    ///
    /// The `action` function informs the application about updates in the
    /// socket (file descriptor) status by doing none, one, or multiple calls to
    /// the socket callback function set with the `socket_function` method. They
    /// update the status with changes since the previous time the callback was
    /// called.
    pub fn action(&self, socket: Socket, events: &Events) -> Result<u32, MultiError> {
        let mut remaining = 0;
        unsafe {
            cvt(curl_sys::curl_multi_socket_action(
                self.raw,
                socket,
                events.bits,
                &mut remaining,
            ))?;
            Ok(remaining as u32)
        }
    }

    /// Inform libcurl that a timeout has expired and sockets should be tested.
    ///
    /// The returned integer will contain the number of running easy handles
    /// within the multi handle. When this number reaches zero, all transfers
    /// are complete/done. When you call `action` on a specific socket and the
    /// counter decreases by one, it DOES NOT necessarily mean that this exact
    /// socket/transfer is the one that completed. Use `messages` to figure out
    /// which easy handle that completed.
    ///
    /// Get the timeout time by calling the `timer_function` method. Your
    /// application will then get called with information on how long to wait
    /// for socket actions at most before doing the timeout action: call the
    /// `timeout` method. You can also use the `get_timeout` function to
    /// poll the value at any given time, but for an event-based system using
    /// the callback is far better than relying on polling the timeout value.
    pub fn timeout(&self) -> Result<u32, MultiError> {
        let mut remaining = 0;
        unsafe {
            cvt(curl_sys::curl_multi_socket_action(
                self.raw,
                curl_sys::CURL_SOCKET_BAD,
                0,
                &mut remaining,
            ))?;
            Ok(remaining as u32)
        }
    }

    /// Get how long to wait for action before proceeding
    ///
    /// An application using the libcurl multi interface should call
    /// `get_timeout` to figure out how long it should wait for socket actions -
    /// at most - before proceeding.
    ///
    /// Proceeding means either doing the socket-style timeout action: call the
    /// `timeout` function, or call `perform` if you're using the simpler and
    /// older multi interface approach.
    ///
    /// The timeout value returned is the duration at this very moment. If 0, it
    /// means you should proceed immediately without waiting for anything. If it
    /// returns `None`, there's no timeout at all set.
    ///
    /// Note: if libcurl returns a `None` timeout here, it just means that
    /// libcurl currently has no stored timeout value. You must not wait too
    /// long (more than a few seconds perhaps) before you call `perform` again.
    pub fn get_timeout(&self) -> Result<Option<Duration>, MultiError> {
        let mut ms = 0;
        unsafe {
            cvt(curl_sys::curl_multi_timeout(self.raw, &mut ms))?;
            if ms == -1 {
                Ok(None)
            } else {
                Ok(Some(Duration::from_millis(ms as u64)))
            }
        }
    }

    /// Block until activity is detected or a timeout passes.
    ///
    /// The timeout is used in millisecond-precision. Large durations are
    /// clamped at the maximum value curl accepts.
    ///
    /// The returned integer will contain the number of internal file
    /// descriptors on which interesting events occured.
    ///
    /// This function is a simpler alternative to using `fdset()` and `select()`
    /// and does not suffer from file descriptor limits.
    ///
    /// # Example
    ///
    /// ```
    /// use curl::multi::Multi;
    /// use std::time::Duration;
    ///
    /// let m = Multi::new();
    ///
    /// // Add some Easy handles...
    ///
    /// while m.perform().unwrap() > 0 {
    ///     m.wait(&mut [], Duration::from_secs(1)).unwrap();
    /// }
    /// ```
    pub fn wait(&self, waitfds: &mut [WaitFd], timeout: Duration) -> Result<u32, MultiError> {
        let timeout_ms = {
            let secs = timeout.as_secs();
            if secs > (i32::max_value() / 1000) as u64 {
                // Duration too large, clamp at maximum value.
                i32::max_value()
            } else {
                secs as i32 * 1000 + timeout.subsec_nanos() as i32 / 1000_000
            }
        };
        unsafe {
            let mut ret = 0;
            cvt(curl_sys::curl_multi_wait(
                self.raw,
                waitfds.as_mut_ptr() as *mut _,
                waitfds.len() as u32,
                timeout_ms,
                &mut ret,
            ))?;
            Ok(ret as u32)
        }
    }

    /// Reads/writes available data from each easy handle.
    ///
    /// This function handles transfers on all the added handles that need
    /// attention in an non-blocking fashion.
    ///
    /// When an application has found out there's data available for this handle
    /// or a timeout has elapsed, the application should call this function to
    /// read/write whatever there is to read or write right now etc.  This
    /// method returns as soon as the reads/writes are done. This function does
    /// not require that there actually is any data available for reading or
    /// that data can be written, it can be called just in case. It will return
    /// the number of handles that still transfer data.
    ///
    /// If the amount of running handles is changed from the previous call (or
    /// is less than the amount of easy handles you've added to the multi
    /// handle), you know that there is one or more transfers less "running".
    /// You can then call `info` to get information about each individual
    /// completed transfer, and that returned info includes `Error` and more.
    /// If an added handle fails very quickly, it may never be counted as a
    /// running handle.
    ///
    /// When running_handles is set to zero (0) on the return of this function,
    /// there is no longer any transfers in progress.
    ///
    /// # Return
    ///
    /// Before libcurl version 7.20.0: If you receive `is_call_perform`, this
    /// basically means that you should call `perform` again, before you select
    /// on more actions. You don't have to do it immediately, but the return
    /// code means that libcurl may have more data available to return or that
    /// there may be more data to send off before it is "satisfied". Do note
    /// that `perform` will return `is_call_perform` only when it wants to be
    /// called again immediately. When things are fine and there is nothing
    /// immediate it wants done, it'll return `Ok` and you need to wait for
    /// "action" and then call this function again.
    ///
    /// This function only returns errors etc regarding the whole multi stack.
    /// Problems still might have occurred on individual transfers even when
    /// this function returns `Ok`. Use `info` to figure out how individual
    /// transfers did.
    pub fn perform(&self) -> Result<u32, MultiError> {
        unsafe {
            let mut ret = 0;
            cvt(curl_sys::curl_multi_perform(self.raw, &mut ret))?;
            Ok(ret as u32)
        }
    }

    /// Extracts file descriptor information from a multi handle
    ///
    /// This function extracts file descriptor information from a given
    /// handle, and libcurl returns its `fd_set` sets. The application can use
    /// these to `select()` on, but be sure to `FD_ZERO` them before calling
    /// this function as curl_multi_fdset only adds its own descriptors, it
    /// doesn't zero or otherwise remove any others. The curl_multi_perform
    /// function should be called as soon as one of them is ready to be read
    /// from or written to.
    ///
    /// If no file descriptors are set by libcurl, this function will return
    /// `Ok(None)`. Otherwise `Ok(Some(n))` will be returned where `n` the
    /// highest descriptor number libcurl set. When `Ok(None)` is returned it
    /// is because libcurl currently does something that isn't possible for
    /// your application to monitor with a socket and unfortunately you can
    /// then not know exactly when the current action is completed using
    /// `select()`. You then need to wait a while before you proceed and call
    /// `perform` anyway.
    ///
    /// When doing `select()`, you should use `get_timeout` to figure out
    /// how long to wait for action. Call `perform` even if no activity has
    /// been seen on the `fd_set`s after the timeout expires as otherwise
    /// internal retries and timeouts may not work as you'd think and want.
    ///
    /// If one of the sockets used by libcurl happens to be larger than what
    /// can be set in an `fd_set`, which on POSIX systems means that the file
    /// descriptor is larger than `FD_SETSIZE`, then libcurl will try to not
    /// set it. Setting a too large file descriptor in an `fd_set` implies an out
    /// of bounds write which can cause crashes, or worse. The effect of NOT
    /// storing it will possibly save you from the crash, but will make your
    /// program NOT wait for sockets it should wait for...
    pub fn fdset2(
        &self,
        read: Option<&mut curl_sys::fd_set>,
        write: Option<&mut curl_sys::fd_set>,
        except: Option<&mut curl_sys::fd_set>,
    ) -> Result<Option<i32>, MultiError> {
        unsafe {
            let mut ret = 0;
            let read = read.map(|r| r as *mut _).unwrap_or(0 as *mut _);
            let write = write.map(|r| r as *mut _).unwrap_or(0 as *mut _);
            let except = except.map(|r| r as *mut _).unwrap_or(0 as *mut _);
            cvt(curl_sys::curl_multi_fdset(
                self.raw, read, write, except, &mut ret,
            ))?;
            if ret == -1 {
                Ok(None)
            } else {
                Ok(Some(ret))
            }
        }
    }

    /// Does nothing and returns `Ok(())`. This method remains for backwards
    /// compatibility.
    ///
    /// This method will be changed to take `self` in a future release.
    #[doc(hidden)]
    #[deprecated(
        since = "0.4.30",
        note = "cannot close safely without consuming self; \
                will be changed or removed in a future release"
    )]
    pub fn close(&self) -> Result<(), MultiError> {
        Ok(())
    }

    /// Get a pointer to the raw underlying CURLM handle.
    pub fn raw(&self) -> *mut curl_sys::CURLM {
        self.raw
    }

    unsafe fn close_impl(&self) -> Result<(), MultiError> {
        cvt(curl_sys::curl_multi_cleanup(self.raw))
    }
}

fn cvt(code: curl_sys::CURLMcode) -> Result<(), MultiError> {
    if code == curl_sys::CURLM_OK {
        Ok(())
    } else {
        Err(MultiError::new(code))
    }
}

impl fmt::Debug for Multi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Multi").field("raw", &self.raw).finish()
    }
}

impl Drop for Multi {
    fn drop(&mut self) {
        let _ = unsafe { self.close_impl() };
    }
}

macro_rules! impl_easy_getters {
    () => {
        impl_easy_getters! {
            time_condition_unmet -> bool,
            effective_url -> Option<&str>,
            effective_url_bytes -> Option<&[u8]>,
            response_code -> u32,
            http_connectcode -> u32,
            filetime -> Option<i64>,
            download_size -> f64,
            content_length_download -> f64,
            total_time -> Duration,
            namelookup_time -> Duration,
            connect_time -> Duration,
            appconnect_time -> Duration,
            pretransfer_time -> Duration,
            starttransfer_time -> Duration,
            redirect_time -> Duration,
            redirect_count -> u32,
            redirect_url -> Option<&str>,
            redirect_url_bytes -> Option<&[u8]>,
            header_size -> u64,
            request_size -> u64,
            content_type -> Option<&str>,
            content_type_bytes -> Option<&[u8]>,
            os_errno -> i32,
            primary_ip -> Option<&str>,
            primary_port -> u16,
            local_ip -> Option<&str>,
            local_port -> u16,
            cookies -> List,
        }
    };

    ($($name:ident -> $ret:ty,)*) => {
        $(
            impl_easy_getters!($name, $ret, concat!(
                "Same as [`Easy2::",
                stringify!($name),
                "`](../easy/struct.Easy2.html#method.",
                stringify!($name),
                ")."
            ));
        )*
    };

    ($name:ident, $ret:ty, $doc:expr) => {
        #[doc = $doc]
        pub fn $name(&mut self) -> Result<$ret, Error> {
            self.easy.$name()
        }
    };
}

impl EasyHandle {
    /// Sets an internal private token for this `EasyHandle`.
    ///
    /// This function will set the `CURLOPT_PRIVATE` field on the underlying
    /// easy handle.
    pub fn set_token(&mut self, token: usize) -> Result<(), Error> {
        unsafe {
            ::cvt(curl_sys::curl_easy_setopt(
                self.easy.raw(),
                curl_sys::CURLOPT_PRIVATE,
                token,
            ))
        }
    }

    impl_easy_getters!();

    /// Unpause reading on a connection.
    ///
    /// Using this function, you can explicitly unpause a connection that was
    /// previously paused.
    ///
    /// A connection can be paused by letting the read or the write callbacks
    /// return `ReadError::Pause` or `WriteError::Pause`.
    ///
    /// The chance is high that you will get your write callback called before
    /// this function returns.
    pub fn unpause_read(&self) -> Result<(), Error> {
        self.easy.unpause_read()
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
    pub fn unpause_write(&self) -> Result<(), Error> {
        self.easy.unpause_write()
    }

    /// Get a pointer to the raw underlying CURL handle.
    pub fn raw(&self) -> *mut curl_sys::CURL {
        self.easy.raw()
    }
}

impl fmt::Debug for EasyHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.easy.fmt(f)
    }
}

impl<H> Easy2Handle<H> {
    /// Acquires a reference to the underlying handler for events.
    pub fn get_ref(&self) -> &H {
        self.easy.get_ref()
    }

    /// Acquires a reference to the underlying handler for events.
    pub fn get_mut(&mut self) -> &mut H {
        self.easy.get_mut()
    }

    /// Same as `EasyHandle::set_token`
    pub fn set_token(&mut self, token: usize) -> Result<(), Error> {
        unsafe {
            ::cvt(curl_sys::curl_easy_setopt(
                self.easy.raw(),
                curl_sys::CURLOPT_PRIVATE,
                token,
            ))
        }
    }

    impl_easy_getters!();

    /// Unpause reading on a connection.
    ///
    /// Using this function, you can explicitly unpause a connection that was
    /// previously paused.
    ///
    /// A connection can be paused by letting the read or the write callbacks
    /// return `ReadError::Pause` or `WriteError::Pause`.
    ///
    /// The chance is high that you will get your write callback called before
    /// this function returns.
    pub fn unpause_read(&self) -> Result<(), Error> {
        self.easy.unpause_read()
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
    pub fn unpause_write(&self) -> Result<(), Error> {
        self.easy.unpause_write()
    }

    /// Get a pointer to the raw underlying CURL handle.
    pub fn raw(&self) -> *mut curl_sys::CURL {
        self.easy.raw()
    }
}

impl<H: fmt::Debug> fmt::Debug for Easy2Handle<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.easy.fmt(f)
    }
}

impl<'multi> Message<'multi> {
    /// If this message indicates that a transfer has finished, returns the
    /// result of the transfer in `Some`.
    ///
    /// If the message doesn't indicate that a transfer has finished, then
    /// `None` is returned.
    ///
    /// Note that the `result*_for` methods below should be preferred as they
    /// provide better error messages as the associated error data on the
    /// handle can be associated with the error type.
    pub fn result(&self) -> Option<Result<(), Error>> {
        unsafe {
            if (*self.ptr).msg == curl_sys::CURLMSG_DONE {
                Some(::cvt((*self.ptr).data as curl_sys::CURLcode))
            } else {
                None
            }
        }
    }

    /// Same as `result`, except only returns `Some` for the specified handle.
    ///
    /// Note that this function produces better error messages than `result` as
    /// it uses `take_error_buf` to associate error information with the
    /// returned error.
    pub fn result_for(&self, handle: &EasyHandle) -> Option<Result<(), Error>> {
        if !self.is_for(handle) {
            return None;
        }
        let mut err = self.result();
        if let Some(Err(e)) = &mut err {
            if let Some(s) = handle.easy.take_error_buf() {
                e.set_extra(s);
            }
        }
        return err;
    }

    /// Same as `result`, except only returns `Some` for the specified handle.
    ///
    /// Note that this function produces better error messages than `result` as
    /// it uses `take_error_buf` to associate error information with the
    /// returned error.
    pub fn result_for2<H>(&self, handle: &Easy2Handle<H>) -> Option<Result<(), Error>> {
        if !self.is_for2(handle) {
            return None;
        }
        let mut err = self.result();
        if let Some(Err(e)) = &mut err {
            if let Some(s) = handle.easy.take_error_buf() {
                e.set_extra(s);
            }
        }
        return err;
    }

    /// Returns whether this easy message was for the specified easy handle or
    /// not.
    pub fn is_for(&self, handle: &EasyHandle) -> bool {
        unsafe { (*self.ptr).easy_handle == handle.easy.raw() }
    }

    /// Same as `is_for`, but for `Easy2Handle`.
    pub fn is_for2<H>(&self, handle: &Easy2Handle<H>) -> bool {
        unsafe { (*self.ptr).easy_handle == handle.easy.raw() }
    }

    /// Returns the token associated with the easy handle that this message
    /// represents a completion for.
    ///
    /// This function will return the token assigned with
    /// `EasyHandle::set_token`. This reads the `CURLINFO_PRIVATE` field of the
    /// underlying `*mut CURL`.
    pub fn token(&self) -> Result<usize, Error> {
        unsafe {
            let mut p = 0usize;
            ::cvt(curl_sys::curl_easy_getinfo(
                (*self.ptr).easy_handle,
                curl_sys::CURLINFO_PRIVATE,
                &mut p,
            ))?;
            Ok(p)
        }
    }
}

impl<'a> fmt::Debug for Message<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Message").field("ptr", &self.ptr).finish()
    }
}

impl Events {
    /// Creates a new blank event bit mask.
    pub fn new() -> Events {
        Events { bits: 0 }
    }

    /// Set or unset the whether these events indicate that input is ready.
    pub fn input(&mut self, val: bool) -> &mut Events {
        self.flag(curl_sys::CURL_CSELECT_IN, val)
    }

    /// Set or unset the whether these events indicate that output is ready.
    pub fn output(&mut self, val: bool) -> &mut Events {
        self.flag(curl_sys::CURL_CSELECT_OUT, val)
    }

    /// Set or unset the whether these events indicate that an error has
    /// happened.
    pub fn error(&mut self, val: bool) -> &mut Events {
        self.flag(curl_sys::CURL_CSELECT_ERR, val)
    }

    fn flag(&mut self, flag: c_int, val: bool) -> &mut Events {
        if val {
            self.bits |= flag;
        } else {
            self.bits &= !flag;
        }
        self
    }
}

impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Events")
            .field("input", &(self.bits & curl_sys::CURL_CSELECT_IN != 0))
            .field("output", &(self.bits & curl_sys::CURL_CSELECT_IN != 0))
            .field("error", &(self.bits & curl_sys::CURL_CSELECT_IN != 0))
            .finish()
    }
}

impl SocketEvents {
    /// Wait for incoming data. For the socket to become readable.
    pub fn input(&self) -> bool {
        self.bits & curl_sys::CURL_POLL_IN == curl_sys::CURL_POLL_IN
    }

    /// Wait for outgoing data. For the socket to become writable.
    pub fn output(&self) -> bool {
        self.bits & curl_sys::CURL_POLL_OUT == curl_sys::CURL_POLL_OUT
    }

    /// Wait for incoming and outgoing data. For the socket to become readable
    /// or writable.
    pub fn input_and_output(&self) -> bool {
        self.bits & curl_sys::CURL_POLL_INOUT == curl_sys::CURL_POLL_INOUT
    }

    /// The specified socket/file descriptor is no longer used by libcurl.
    pub fn remove(&self) -> bool {
        self.bits & curl_sys::CURL_POLL_REMOVE == curl_sys::CURL_POLL_REMOVE
    }
}

impl fmt::Debug for SocketEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Events")
            .field("input", &self.input())
            .field("output", &self.output())
            .field("remove", &self.remove())
            .finish()
    }
}

impl WaitFd {
    /// Constructs an empty (invalid) WaitFd.
    pub fn new() -> WaitFd {
        WaitFd {
            inner: curl_sys::curl_waitfd {
                fd: 0,
                events: 0,
                revents: 0,
            },
        }
    }

    /// Set the file descriptor to wait for.
    pub fn set_fd(&mut self, fd: Socket) {
        self.inner.fd = fd;
    }

    /// Indicate that the socket should poll on read events such as new data
    /// received.
    ///
    /// Corresponds to `CURL_WAIT_POLLIN`.
    pub fn poll_on_read(&mut self, val: bool) -> &mut WaitFd {
        self.flag(curl_sys::CURL_WAIT_POLLIN, val)
    }

    /// Indicate that the socket should poll on high priority read events such
    /// as out of band data.
    ///
    /// Corresponds to `CURL_WAIT_POLLPRI`.
    pub fn poll_on_priority_read(&mut self, val: bool) -> &mut WaitFd {
        self.flag(curl_sys::CURL_WAIT_POLLPRI, val)
    }

    /// Indicate that the socket should poll on write events such as the socket
    /// being clear to write without blocking.
    ///
    /// Corresponds to `CURL_WAIT_POLLOUT`.
    pub fn poll_on_write(&mut self, val: bool) -> &mut WaitFd {
        self.flag(curl_sys::CURL_WAIT_POLLOUT, val)
    }

    fn flag(&mut self, flag: c_short, val: bool) -> &mut WaitFd {
        if val {
            self.inner.events |= flag;
        } else {
            self.inner.events &= !flag;
        }
        self
    }

    /// After a call to `wait`, returns `true` if `poll_on_read` was set and a
    /// read event occured.
    pub fn received_read(&self) -> bool {
        self.inner.revents & curl_sys::CURL_WAIT_POLLIN == curl_sys::CURL_WAIT_POLLIN
    }

    /// After a call to `wait`, returns `true` if `poll_on_priority_read` was set and a
    /// priority read event occured.
    pub fn received_priority_read(&self) -> bool {
        self.inner.revents & curl_sys::CURL_WAIT_POLLPRI == curl_sys::CURL_WAIT_POLLPRI
    }

    /// After a call to `wait`, returns `true` if `poll_on_write` was set and a
    /// write event occured.
    pub fn received_write(&self) -> bool {
        self.inner.revents & curl_sys::CURL_WAIT_POLLOUT == curl_sys::CURL_WAIT_POLLOUT
    }
}

#[cfg(unix)]
impl From<pollfd> for WaitFd {
    fn from(pfd: pollfd) -> WaitFd {
        let mut events = 0;
        if pfd.events & POLLIN == POLLIN {
            events |= curl_sys::CURL_WAIT_POLLIN;
        }
        if pfd.events & POLLPRI == POLLPRI {
            events |= curl_sys::CURL_WAIT_POLLPRI;
        }
        if pfd.events & POLLOUT == POLLOUT {
            events |= curl_sys::CURL_WAIT_POLLOUT;
        }
        WaitFd {
            inner: curl_sys::curl_waitfd {
                fd: pfd.fd,
                events: events,
                revents: 0,
            },
        }
    }
}

impl fmt::Debug for WaitFd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WaitFd")
            .field("fd", &self.inner.fd)
            .field("events", &self.inner.fd)
            .field("revents", &self.inner.fd)
            .finish()
    }
}
