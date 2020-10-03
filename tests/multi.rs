#![cfg(unix)]

extern crate curl;
extern crate mio;
extern crate mio_misc;

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::time::Duration;

use curl::easy::{Easy, List};
use curl::multi::Multi;

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

use server::Server;
mod server;

#[test]
fn smoke() {
    let m = Multi::new();
    let mut e = Easy::new();

    let s = Server::new();
    s.receive(
        "\
         GET / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         \r\n",
    );
    s.send("HTTP/1.1 200 OK\r\n\r\n");

    t!(e.url(&s.url("/")));
    let _e = t!(m.add(e));
    while t!(m.perform()) > 0 {
        t!(m.wait(&mut [], Duration::from_secs(1)));
    }
}

#[test]
fn smoke2() {
    let m = Multi::new();

    let s1 = Server::new();
    s1.receive(
        "\
         GET / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         \r\n",
    );
    s1.send("HTTP/1.1 200 OK\r\n\r\n");

    let s2 = Server::new();
    s2.receive(
        "\
         GET / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         \r\n",
    );
    s2.send("HTTP/1.1 200 OK\r\n\r\n");

    let mut e1 = Easy::new();
    t!(e1.url(&s1.url("/")));
    let _e1 = t!(m.add(e1));
    let mut e2 = Easy::new();
    t!(e2.url(&s2.url("/")));
    let _e2 = t!(m.add(e2));

    while t!(m.perform()) > 0 {
        t!(m.wait(&mut [], Duration::from_secs(1)));
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
    use curl::multi::{Events, Socket, SocketEvents};

    #[derive(Debug)]
    enum Message {
        Timeout(Option<Duration>),
        Wait(Socket, SocketEvents, usize),
    }

    let mut m = Multi::new();
    let mut poll = t!(mio::Poll::new());
    let q = mio_misc::queue::NotificationQueue::new(std::sync::Arc::new(
        mio::Waker::new(poll.registry(), mio::Token(0)).unwrap(),
    ));
    let (tx, rx) =
        mio_misc::channel::channel(std::sync::Arc::new(q), mio_misc::NotificationId::gen_next());
    let tx2 = tx.clone();
    t!(m.socket_function(move |socket, events, token| {
        t!(tx2.send(Message::Wait(socket, events, token)));
    }));
    t!(m.timer_function(move |dur| {
        t!(tx.send(Message::Timeout(dur)));
        true
    }));

    let s = Server::new();
    s.receive(&format!(
        "\
         PUT / HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         Content-Length: 131072\r\n\
         \r\n\
         {}\n",
        vec!["a"; 128 * 1024 - 1].join("")
    ));
    s.send(
        "\
         HTTP/1.1 200 OK\r\n\
         \r\n",
    );

    let mut data = vec![b'a'; 128 * 1024 - 1];
    data.push(b'\n');
    let mut data = Cursor::new(data);
    let mut list = List::new();
    t!(list.append("Expect:"));
    let mut h = Easy::new();
    t!(h.url(&s.url("/")));
    t!(h.put(true));
    t!(h.read_function(move |buf| Ok(data.read(buf).unwrap())));
    t!(h.in_filesize(128 * 1024));
    t!(h.upload(true));
    t!(h.http_headers(list));

    let e = t!(m.add(h));

    assert!(t!(m.perform()) > 0);
    let mut next_token = 1;
    let mut token_map = HashMap::new();
    let mut cur_timeout = None;
    let mut events = mio::Events::with_capacity(128);
    let mut running = true;

    while running {
        t!(poll.poll(&mut events, cur_timeout));

        if events.is_empty() {
            if t!(m.timeout()) == 0 {
                running = false;
            }
        }

        for event in events.iter() {
            while event.token() == mio::Token(0) {
                match rx.try_recv() {
                    Ok(Message::Timeout(dur)) => cur_timeout = dur,
                    Ok(Message::Wait(socket, events, token)) => {
                        let mut evented = mio::unix::SourceFd(&socket);
                        if events.remove() {
                            token_map.remove(&token).unwrap();
                        } else {
                            let e = {
                                if events.input() & events.output() {
                                    mio::Interest::READABLE | mio::Interest::WRITABLE
                                } else if events.input() {
                                    mio::Interest::READABLE
                                } else if events.output() {
                                    mio::Interest::WRITABLE
                                } else {
                                    continue;
                                }
                            };
                            if token == 0 {
                                let token = next_token;
                                next_token += 1;
                                t!(m.assign(socket, token));
                                token_map.insert(token, socket);
                                t!(poll.registry().register(&mut evented, mio::Token(token), e));
                            } else {
                                t!(poll
                                    .registry()
                                    .reregister(&mut evented, mio::Token(token), e));
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            if event.token() == mio::Token(0) {
                continue;
            }

            let token = event.token();
            let socket = token_map[&token.into()];
            let mut e = Events::new();
            if event.is_readable() {
                e.input(true);
            }
            if event.is_writable() {
                e.output(true);
            }
            if event.is_error() {
                e.error(true);
            }
            let remaining = t!(m.action(socket, &e));
            if remaining == 0 {
                running = false;
            }
        }
    }

    let mut done = 0;
    m.messages(|m| {
        m.result().unwrap().unwrap();
        done += 1;
    });
    assert_eq!(done, 1);

    let mut e = t!(m.remove(e));
    assert_eq!(t!(e.response_code()), 200);
}

// Tests passing raw file descriptors to Multi::wait. The test is limited to Linux only as the
// semantics of the underlying poll(2) system call used by curl apparently differ on other
// platforms, making the test fail.
#[cfg(target_os = "linux")]
#[test]
fn waitfds() {
    use curl::multi::WaitFd;
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    let filenames = ["/dev/null", "/dev/zero", "/dev/urandom"];
    let files: Vec<File> = filenames
        .iter()
        .map(|filename| File::open(filename).unwrap())
        .collect();
    let mut waitfds: Vec<WaitFd> = files
        .iter()
        .map(|f| {
            let mut waitfd = WaitFd::new();
            waitfd.set_fd(f.as_raw_fd());
            waitfd.poll_on_read(true);
            waitfd
        })
        .collect();

    let m = Multi::new();
    let events = t!(m.wait(&mut waitfds, Duration::from_secs(1)));
    assert_eq!(events, 3);
    for waitfd in waitfds {
        assert!(waitfd.received_read());
    }
}
