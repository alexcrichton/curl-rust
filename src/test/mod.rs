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
  ($($e:expr),*) => (server::SendBytes(bytes!($($e),*)));
)

macro_rules! recv(
  ($($e:expr),*) => (server::ReceiveBytes(bytes!($($e),*)));
)

macro_rules! wait(
  ($dur:expr) => (server::Wait($dur));
)

// mod simple;
mod get;
mod post;
mod keep_alive;
