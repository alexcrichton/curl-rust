use std::io::SeekFrom;

use crate::easy::{ReadError, SeekResult};

/// A trait to provide data for a MIME part.
pub trait PartDataHandler {
    /// Read callback for data uploads.
    ///
    /// This callback function gets called by libcurl as soon as it needs to
    /// read data in order to send it to the peer.
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
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError>;

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
    fn seek(&mut self, whence: SeekFrom) -> SeekResult {
        let _ = whence; // ignore unused
        SeekResult::CantSeek
    }
}
