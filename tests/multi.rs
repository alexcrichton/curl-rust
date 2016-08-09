#![cfg(unix)]

extern crate curl;
extern crate mio;

use std::collections::HashMap;
use std::io::{Read, Cursor};
use std::time::Duration;

use curl::easy::{Easy, List};
use curl::multi::Multi;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

use server::Server;
mod server;

#[test]
fn smoke() {
    let m = Multi::new();
    let mut e = Easy::new();

    let s = Server::new();
    s.receive("\
GET / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
\r\n");
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    t!(e.url(&s.url("/")));
    let _e = t!(m.add(e));
    while t!(m.perform()) > 0 {
        // ...
    }
}

#[test]
fn smoke2() {
    let m = Multi::new();

    let s1 = Server::new();
    s1.receive("\
GET / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
\r\n");
    s1.send("HTTP/1.1 200 OK\r\n\r\n");

    let s2 = Server::new();
    s2.receive("\
GET / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
\r\n");
    s2.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut e1 = Easy::new();
    t!(e1.url(&s1.url("/")));
    let _e1 = t!(m.add(e1));
    let mut e2 = Easy::new();
    t!(e2.url(&s2.url("/")));
    let _e2 = t!(m.add(e2));

    while t!(m.perform()) > 0 {
        // ...
    }

    let mut done = 0;
    m.messages(|msg| {
        msg.result().unwrap().unwrap();
        done += 1;
    });
    assert_eq!(done, 2);
}

#[test]
fn upload_lots() {
    use curl::multi::{Socket, SocketEvents, Events};

    enum Message {
        Timeout(Option<Duration>),
        Wait(Socket, SocketEvents, usize),
    }

    let mut m = Multi::new();
    let mut l = t!(mio::EventLoop::new());
    let io = l.channel();
    t!(m.socket_function(move |socket, events, token| {
        t!(io.send(Message::Wait(socket, events, token)));
    }));
    let io = l.channel();
    t!(m.timer_function(move |dur| {
        t!(io.send(Message::Timeout(dur)));
        true
    }));

    let s = Server::new();
    s.receive(&format!("\
PUT / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
Accept: */*\r\n\
Content-Length: 131072\r\n\
\r\n\
{}\n", vec!["a"; 128 * 1024 - 1].join("")));
    s.send("\
HTTP/1.1 200 OK\r\n\
\r\n");

    let mut data = vec![b'a'; 128 * 1024 - 1];
    data.push(b'\n');
    let mut data = Cursor::new(data);
    let mut list = List::new();
    t!(list.append("Expect:"));
    let mut h = Easy::new();
    t!(h.url(&s.url("/")));
    t!(h.put(true));
    t!(h.read_function(move |buf| {
        Ok(data.read(buf).unwrap())
    }));
    t!(h.in_filesize(128 * 1024));
    t!(h.upload(true));
    t!(h.http_headers(list));

    let e = t!(m.add(h));

    assert!(t!(m.perform()) > 0);
    t!(l.run(&mut Handler {
        multi: &m,
        cur_timeout: None,
        next_token: 1,
        token_map: HashMap::new(),
    }));

    let mut done = 0;
    m.messages(|m| {
        m.result().unwrap().unwrap();
        done += 1;
    });
    assert_eq!(done, 1);

    let mut e = t!(m.remove(e));
    assert_eq!(t!(e.response_code()), 200);

    struct Handler<'a> {
        multi: &'a Multi,
        cur_timeout: Option<mio::Timeout>,
        next_token: usize,
        token_map: HashMap<usize, Socket>,
    }

    impl<'a> mio::Handler for Handler<'a> {
        type Timeout = ();
        type Message = Message;

        fn ready(&mut self,
                 l: &mut mio::EventLoop<Handler<'a>>,
                 token: mio::Token,
                 events: mio::EventSet) {
            let socket = self.token_map[&token.as_usize()];
            let mut e = Events::new();
            if events.is_readable() {
                e.input(true);
            }
            if events.is_writable() {
                e.output(true);
            }
            if events.is_error() {
                e.error(true);
            }
            let remaining = t!(self.multi.action(socket, &e));
            if remaining == 0 {
                l.shutdown();
            }
        }

        fn timeout(&mut self,
                   l: &mut mio::EventLoop<Handler<'a>>,
                   _msg: ()) {
            if t!(self.multi.timeout()) == 0 {
                l.shutdown();
            }
        }

        fn notify(&mut self,
                  l: &mut mio::EventLoop<Handler<'a>>,
                  msg: Message) {
            match msg {
                Message::Timeout(dur) => {
                    if let Some(t) = self.cur_timeout.take() {
                        l.clear_timeout(t);
                    }
                    if let Some(dur) = dur {
                        let ms = dur.as_secs() * 1_000 +
                                 (dur.subsec_nanos() / 1_000_000) as u64;
                        self.cur_timeout = Some(t!(l.timeout_ms((), ms)));
                    }
                }
                Message::Wait(socket, events, token) => {
                    let evented = mio::unix::EventedFd(&socket);
                    if events.remove() {
                        t!(l.deregister(&evented));
                        self.token_map.remove(&token).unwrap();
                    } else {
                        let mut e = mio::EventSet::none();
                        if events.input() {
                            e = e | mio::EventSet::readable();
                        }
                        if events.output() {
                            e = e | mio::EventSet::writable();
                        }
                        if token == 0 {
                            let token = self.next_token;
                            self.next_token += 1;
                            t!(self.multi.assign(socket, token));
                            self.token_map.insert(token, socket);
                            t!(l.register(&evented,
                                          mio::Token(token),
                                          e,
                                          mio::PollOpt::level()));
                        } else {
                            t!(l.reregister(&evented,
                                            mio::Token(token),
                                            e,
                                            mio::PollOpt::level()));
                        }
                    }
                }
            }
        }
    }
}
