// #![macro_escape]

mod server;

macro_rules! server(
  ($($ops:expr),+) => (server::setup(ops!($($ops),+)));
)

macro_rules! ops(
  ($op:expr) => (server::OpSequence::new($op));
  ($op:expr, $($res:expr),+) => (
    server::OpSequence::concat($op, ops!($($res),+))
  );
)

macro_rules! send(
  ($e:expr) => (server::SendBytes($e));
)

macro_rules! recv(
  ($e:expr) => (server::ReceiveBytes($e));
)

macro_rules! wait(
  ($dur:expr) => (server::Wait($dur));
)

// mod simple;
mod get;
mod head;
mod post;
mod put;
mod delete;
mod keep_alive;
