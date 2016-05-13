use std::any::Any;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};

thread_local!(static LAST_ERROR: RefCell<Option<Box<Any + Send>>> = {
    RefCell::new(None)
});

pub fn catch<T, F: FnOnce() -> T>(f: F) -> Option<T> {
    if LAST_ERROR.with(|slot| slot.borrow().is_some()) {
        return None
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
    if let Some(t) = LAST_ERROR.with(|slot| slot.borrow_mut().take()) {
        panic::resume_unwind(t)
    }
}
