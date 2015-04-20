use std::collections::HashSet;
use std::io::prelude::*;
use std::iter::repeat;
use std::net::{TcpStream, TcpListener};
use std::str;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use self::Op::{SendBytes, ReceiveBytes, Wait, Shutdown};

// Global handle to the running test HTTP server
thread_local!(static HANDLE: Handle = start_server());

// Setup an op sequence with the test HTTP server
pub fn setup(ops: OpSequence) -> OpSequenceResult {
    // Setup a channel to receive the response on
    let (tx, rx) = channel();

    // Send the op sequence to the server task
    HANDLE.with(move |h| {
        h.send(ops, tx);
    });

    OpSequenceResult::new(rx)
}

pub fn url(path: &str) -> String {
    format!("http://localhost:{}{}", port(), path)
}

fn port() -> usize {
    HANDLE.with(|h| {
        h.port()
    })
}

/* Handle to the running HTTP server task. Communication with the server
 * happens over channels.
 */
struct Handle {
    port: u16,
    tx: Sender<(OpSequence, Sender<Result<(),String>>)>
}

/* Operations for the test server to perform:
 * - Send some bytes
 * - Expect to receive bytes
 * - Wait for a certain amount of time
 * - Shutdown the server (allows a clean exit at the end of the tests)
 */
#[derive(Clone,PartialEq,Debug)]
pub enum Op {
    SendBytes(&'static [u8]),
    ReceiveBytes(&'static [u8]),
    Wait(usize),
    Shutdown
}

/* An ordered sequence of operations for the HTTP server to perform
*/
#[derive(Debug)]
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
        ops.extend(seq.ops.iter().cloned());
        OpSequence { ops: ops }
    }

    pub fn is_shutdown(&self) -> bool {
        self.ops.len() == 1 && self.ops[0] == Shutdown
    }

    pub fn apply(&self, sock: &mut TcpStream, port: usize)
                 -> Result<(), String> {
        for op in self.ops.iter() {
            match op {
                &SendBytes(b) => {
                    let b = insert_port(b, port);
                    match sock.write_all(&b) {
                        Ok(_) => {}
                        Err(e) => return Err(e.to_string())
                    }
                }
                &ReceiveBytes(b) => {
                    let b = insert_port(b, port);
                    let mut rem = b.len();
                    let mut act = repeat(0u8).take(rem).collect::<Vec<_>>();

                    while rem > 0 {
                        match sock.read(&mut act[b.len() - rem..]) {
                            Ok(i) => rem = rem - i,
                            Err(e) => {
                                debug!("aborting due to error; error={}; \
                                        remaining={}; bytes=\n{}", e, rem,
                                       to_debug_str(&act));
                                return Err(e.to_string())
                            }
                        }
                    }

                    debug!("server received bytes; rem={}; bytes=\n{}\n~~~~~~~~~~~~~~",
                           rem, to_debug_str(&act));

                    let req1 = parse_request(&b);
                    let req2 = parse_request(&act);

                    if req1 != req2 {
                        return Err(format!(
                                "received unexpected byte sequence.\n\n\
                                 Expected:\n{}\n\nReceived:\n{}",
                                to_debug_str(&b), to_debug_str(&act)));
                    }
                }
                &Wait(ms) => thread::sleep_ms(ms as u32),
                &Shutdown => {
                    return Err("Shutdown must be sent on its own".to_string())
                }
            }
        }

        return Ok(());

        fn insert_port(bytes: &'static [u8], port: usize) -> Vec<u8> {
            let s = str::from_utf8(bytes).unwrap();
            let p = port.to_string();
            s.replace("{PORT}", &p).into_bytes()
        }

        fn parse_request<'a>(req: &'a [u8]) -> (&'a [u8],
                                                HashSet<&'a [u8]>,
                                                &'a [u8]) {
            let mut start = None;
            let mut headers = HashSet::new();
            let mut taken = 0;

            for part in req.split(|a| *a == b'\n') {
                taken += part.len() + 1;

                if start.is_none() {
                    start = Some(part);
                } else if part.len() == 1 {
                    break;
                } else {
                    headers.insert(part);
                }
            }

            if taken > req.len() {
                taken = req.len();
            }

            (start.unwrap(), headers, &req[taken..])
        }
    }
}

fn to_debug_str(bytes: &[u8]) -> String {
    let mut ret = String::new();

    for b in bytes.iter() {
        let b = *b as char;

        if b >= ' ' && b <= '~' {
            ret.push(b);
        }
        else if b == '\n' {
            ret.push_str("\\n\n");
        }
        else if b == '\r' {
            ret.push_str("\\r");
        }
        else {
            ret.push('?');
        }
    }

    ret
}

impl Handle {
    fn new(tx: Sender<(OpSequence, Sender<Result<(),String>>)>, port: u16)
           -> Handle {
        Handle { tx: tx, port: port }
    }

    fn send(&self, ops: OpSequence, resp: Sender<Result<(),String>>) {
        self.tx.send((ops, resp)).unwrap();
    }

    fn port(&self) -> usize {
        self.port as usize
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let (tx, _) = channel();
        self.send(OpSequence::new(Shutdown), tx);
    }
}

impl OpSequenceResult {
    pub fn new(rx: Receiver<Result<(),String>>) -> OpSequenceResult {
        OpSequenceResult { rx: rx }
    }

    pub fn assert(&self) {
        match self.rx.recv().unwrap() {
            Ok(_) => {}
            Err(e) => panic!("http exchange did not proceed as expected: {}", e)
        }
    }
}

fn start_server() -> Handle {
    let (ops_tx, ops_rx) = channel();

    let srv = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = srv.local_addr().unwrap().port();

    thread::spawn(move || {
        loop {
            let (ops, resp_tx): (OpSequence, Sender<Result<(),String>>) =
                    ops_rx.recv().unwrap();

            if ops.is_shutdown() {
                return;
            }

            let mut sock = match srv.accept() {
                Ok(s) => s.0,
                Err(e) => {
                    resp_tx.send(Err(format!("server accept err: {}", e))).unwrap();
                    return;
                }
            };

            resp_tx.send(ops.apply(&mut sock, port as usize)).unwrap();
        }
    });

    Handle::new(ops_tx, port)
}
