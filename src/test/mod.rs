mod server;

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

mod get;
mod head;
mod post;
mod proxy;
mod put;
mod delete;
mod keep_alive;
