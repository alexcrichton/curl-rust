pub use self::handle::{Handle,Request,Method};

pub use self::multi_handle::MultiHandle;
pub use self::response::{Headers,Response};

pub mod body;
pub mod handle;
pub mod multi_handle;
pub mod header;
mod response;



#[inline]
pub fn handle() -> Handle {
    Handle::new()
}

#[inline]
pub fn multi_handle() -> MultiHandle {
    MultiHandle::new()
}
