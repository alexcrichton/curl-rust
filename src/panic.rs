struct Bomb {
    armed: bool,
}

impl Drop for Bomb {
    fn drop(&mut self) {
        if self.armed {
            panic!("cannot panic in a callback in libcurl, panicking again to \
                    abort the process safely")
        }
    }
}

pub fn catch<R, F: FnOnce() -> R>(f: F) -> Option<R> {
    let mut bomb = Bomb { armed: true };
    let ret = f();
    bomb.armed = false;
    Some(ret)
}

pub fn propagate() {
}
