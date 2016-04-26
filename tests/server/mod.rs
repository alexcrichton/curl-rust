use std::collections::HashSet;
use std::net::{TcpListener, SocketAddr, TcpStream};
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::BufReader;

pub struct Server {
    messages: Option<Sender<Message>>,
    addr: SocketAddr,
    thread: Option<thread::JoinHandle<()>>,
}

enum Message {
    Read(String),
    Write(String),
}

fn run(listener: &TcpListener, rx: &Receiver<Message>) {
    let mut socket = BufReader::new(listener.accept().unwrap().0);
    for msg in rx.iter() {
        match msg {
            Message::Read(ref expected) => {
                let mut expected = &expected[..];
                let mut expected_headers = HashSet::new();
                while let Some(i) = expected.find("\n") {
                    let line = &expected[..i + 1];
                    expected = &expected[i + 1..];
                    if line == "\r" || line == "" {
                        break
                    }
                    expected_headers.insert(line);
                }
                if expected.len() > 0 {
                    assert!(expected_headers.insert(expected));
                }

                while expected_headers.len() > 0 {
                    let mut actual = String::new();
                    t!(socket.read_line(&mut actual));
                    if !expected_headers.remove(&actual[..]) {
                        panic!("unexpected header: {:?}", actual);
                    }
                }
                for header in expected_headers {
                    panic!("expected header but not found: {:?}", header);
                }

                let mut buf = vec![0; expected.len()];
                if buf.len() > 0 {
                    t!(socket.read_exact(&mut buf));
                    assert_eq!(buf, expected.as_bytes());
                }
            }
            Message::Write(ref to_write) => {
                t!(socket.get_mut().write_all(to_write.as_bytes()));
                return
            }
        }
    }

    let mut dst = Vec::new();
    t!(socket.read_to_end(&mut dst));
    assert!(dst.len() == 0);
}

impl Server {
    pub fn new() -> Server {
        let listener = t!(TcpListener::bind("127.0.0.1:0"));
        let addr = t!(listener.local_addr());
        let (tx, rx) = channel();
        let thread = thread::spawn(move || run(&listener, &rx));
        Server {
            messages: Some(tx),
            addr: addr,
            thread: Some(thread),
        }
    }

    pub fn receive(&self, msg: &str) {
        let msg = msg.replace("$PORT", &self.addr.port().to_string());
        self.msg(Message::Read(msg));
    }

    pub fn send(&self, msg: &str) {
        let msg = msg.replace("$PORT", &self.addr.port().to_string());
        self.msg(Message::Write(msg));
    }

    fn msg(&self, msg: Message) {
        t!(self.messages.as_ref().unwrap().send(msg));
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        drop(TcpStream::connect(&self.addr));
        drop(self.messages.take());
        let res = self.thread.take().unwrap().join();
        if !thread::panicking() {
            t!(res);
        } else if let Err(e) = res {
            println!("child server thread also failed: {:?}", e);
        }
    }
}
