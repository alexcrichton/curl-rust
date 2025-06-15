use std::fmt;

use super::handle::EasyData;
use super::handler::Inner;
#[cfg(doc)]
use super::{Easy, Handler};

/// Provides access to the handle inside [`Handler::write`] callback.
pub struct WriteContext2<H: ?Sized> {
    inner: *mut Inner<H>,
}

/// Provides access to the handle inside [`Easy::write_function`] callback.
#[repr(transparent)]
pub struct WriteContext(WriteContext2<EasyData>);

impl<H: ?Sized> WriteContext2<H> {
    /// Returns the raw Easy pointer.
    #[inline]
    pub fn raw(&self) -> *mut curl_sys::CURL {
        // Safety: make sure not to borrow `inner` that would be an alias to the inner handle
        // activated in a Handler callback.
        unsafe { *std::ptr::addr_of!((*self.inner).handle) }
    }
}

impl WriteContext {
    /// Returns the raw Easy pointer.
    #[inline]
    pub fn raw(&self) -> *mut curl_sys::CURL {
        self.0.raw()
    }

    pub(super) fn from_mut(inner: &mut WriteContext2<EasyData>) -> &mut Self {
        // Safety: `inner` has repr transparent over WriteContext2<EasyData>.
        unsafe { std::mem::transmute::<&mut WriteContext2<EasyData>, &mut Self>(inner) }
    }
}

impl<H: ?Sized> fmt::Debug for WriteContext2<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteContext2").finish()
    }
}

impl fmt::Debug for WriteContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteContext").finish()
    }
}
