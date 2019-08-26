use std::any::Any;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};

thread_local!(static LAST_ERROR: RefCell<Option<Box<dyn Any + Send>>> = {
    RefCell::new(None)
});

pub fn catch<T, F: FnOnce() -> T>(f: F) -> Option<T> {
    match LAST_ERROR.try_with(|slot| slot.borrow().is_some()) {
        Ok(true) => return None,
        Ok(false) => {}
        // we're in thread shutdown, so we're for sure not panicking and
        // panicking again will abort, so no need to worry!
        Err(_) => {}
    }

    // Note that `AssertUnwindSafe` is used here as we prevent reentering
    // arbitrary code due to the `LAST_ERROR` check above plus propagation of a
    // panic after we return back to user code from C.
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(ret) => Some(ret),
        Err(e) => {
            LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(e));
            None
        }
    }
}

pub fn propagate() {
    if let Ok(Some(t)) = LAST_ERROR.try_with(|slot| slot.borrow_mut().take()) {
        panic::resume_unwind(t)
    }
}
