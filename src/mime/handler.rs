use std::io::SeekFrom;

use crate::easy::{ReadError, SeekResult};

pub trait PartDataHandler {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError>;
    fn seek(&mut self, whence: SeekFrom) -> SeekResult {
        let _ = whence; // ignore unused
        SeekResult::CantSeek
    }
}
