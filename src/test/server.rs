use std::cell::RefCell;
use std::io::IoResult;
use std::io::net::tcp::{TcpAcceptor,TcpListener,TcpStream};
use std::io::{Acceptor, Listener};
use std::io::stdio::stdout;
use std::io::timer;

// Global handle to the running test HTTP server
local_data_key!(handle: Handle)

// Setup an op sequence with the test HTTP server
pub fn setup(ops: OpSequence) -> OpSequenceResult {
  // If the server si not started
  ensure_server_started();

  // Setup a channel to receive the response on
  let (tx, rx) = channel();

  // Send the op sequence to the server task
  handle.get().unwrap().send(ops, tx);

  OpSequenceResult::new(rx)
}

/* Handle to the running HTTP server task. Communication with the server
 * happesn over channels.
 */
struct Handle {
  tx: Sender<(OpSequence, Sender<Result<(),String>>)>
}

/* Operations for the test server to perform:
 * - Send some bytes
 * - Expect to receive bytes
 * - Wait for a certain amount of time
 * - Shutdown the server (allows a clean exit at the end of the tests)
 */
#[deriving(Clone,Eq,Show)]
pub enum Op {
  SendBytes(&'static [u8]),
  ReceiveBytes(&'static [u8]),
  Wait(uint),
  Shutdown
}

/* An ordered sequence of operations for the HTTP server to perform
 */
pub struct OpSequence {
  ops: Vec<Op>
}

/* Represents the completion of the of the op sequence by the HTTP
 * server.
 */
pub struct OpSequenceResult {
  rx: Receiver<Result<(),String>>
}

impl OpSequence {
  pub fn new(op: Op) -> OpSequence {
    OpSequence { ops: vec!(op) }
  }

  pub fn concat(op: Op, seq: OpSequence) -> OpSequence {
    let mut ops = vec!(op);
    ops.push_all(seq.ops.as_slice());
    OpSequence { ops: ops }
  }

  pub fn is_shutdown(&self) -> bool {
    self.ops.len() == 1 && self.ops.get(0) == &Shutdown
  }

  pub fn apply(&self, sock: &mut TcpStream) -> Result<(), String> {
    for op in self.ops.iter() {
      println!("applying op={}", op);
      match op {
        &SendBytes(b) => {
          stdout().write_str("sending bytes\n");
          sock.write(b);
        }
        &ReceiveBytes(b) => {
          println!("b={}", sock.read_byte());
          let mut rem = b.len();
          let mut act = Vec::from_elem(rem, 0u8);

          while rem > 0 {
            println!("reading; rem={}; {}", rem, act.mut_slice_from(b.len() - rem).len());

            match sock.read(act.mut_slice_from(b.len() - rem)) {
              Ok(i) => rem = rem - i,
              Err(e) => {
                println!(" read err {}", e);
                return Err(e.desc.to_string())
              }
            }
          }

          if b != act.as_slice() {
            return Err(to_str(act.as_slice()));
            // return Err("did not receive expected bytes".to_string());
          }
        }
        &Wait(ms) => { timer::sleep(ms as u64) }
        &Shutdown => return Err("Shutdown must be sent on its own".to_string())
      }
    }

    Ok(())
  }
}

fn to_str(b: &[u8]) -> String {
  String::new()
}

impl Handle {
  fn new(tx: Sender<(OpSequence, Sender<Result<(),String>>)>) -> Handle {
    Handle { tx: tx }
  }

  fn send(&self, ops: OpSequence, resp: Sender<Result<(),String>>) {
    self.tx.send((ops, resp));
  }
}

impl Drop for Handle {
  fn drop(&mut self) {
    let (tx, rx) = channel();
    self.send(OpSequence::new(Shutdown), tx);
    rx.recv();
  }
}

impl OpSequenceResult {
  pub fn new(rx: Receiver<Result<(),String>>) -> OpSequenceResult {
    OpSequenceResult { rx: rx }
  }

  pub fn assert(&self) {
    match self.rx.recv() {
      Ok(_) => {}
      Err(e) => fail!("http exchange did not proceed as expected: {}", e)
    }
  }
}

fn ensure_server_started() {
  if handle.get().is_none() {
    handle.replace(Some(start_server()));
  }
}

fn start_server() -> Handle {
  let (ops_tx, ops_rx) = channel();
  let (ini_tx, ini_rx) = channel();

  spawn(proc() {
    let mut srv = TcpListener::bind("127.0.0.1", 8482).unwrap().listen().unwrap();

    ini_tx.send(true);

    loop {
      let (ops, resp_tx): (OpSequence, Sender<Result<(),String>>) = ops_rx.recv();

      if ops.is_shutdown() {
        resp_tx.send(Ok(()));
        return;
      }

      let mut sock = match srv.accept() {
        Ok(s) => s,
        Err(e) => {
          resp_tx.send(Err(format!("server accept err: {}", e)));
          return;
        }
      };

      resp_tx.send(ops.apply(&mut sock));
    }
  });

  // Wait until the server is listening
  ini_rx.recv();

  Handle::new(ops_tx)
}
