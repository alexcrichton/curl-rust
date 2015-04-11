extern crate curl;
extern crate env_logger;

#[macro_use]
extern crate log;

macro_rules! server {
    ($($ops:expr),+) => (server::setup(ops!($($ops),+)));
}

macro_rules! ops {
    ($op:expr) => (server::OpSequence::new($op));
    ($op:expr, $($res:expr),+) => (
        server::OpSequence::concat($op, ops!($($res),+))
    );
}

macro_rules! send {
    ($e:expr) => (server::Op::SendBytes($e));
}

macro_rules! recv {
    ($e:expr) => (server::Op::ReceiveBytes($e));
}

macro_rules! wait {
    ($dur:expr) => (server::Op::Wait($dur));
}

mod server;
mod test_delete;
mod test_get;
mod test_head;
mod test_keep_alive;
mod test_patch;
mod test_post;
mod test_proxy;
mod test_put;
