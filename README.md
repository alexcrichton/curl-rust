# curl-rust

libcurl bindings for Rust

[![Build Status](https://travis-ci.org/alexcrichton/curl-rust.svg?branch=master)](https://travis-ci.org/alexcrichton/curl-rust)
[![Build status](https://ci.appveyor.com/api/projects/status/lx98wtbxhhhajpr9?svg=true)](https://ci.appveyor.com/project/alexcrichton/curl-rust)

[Documentation](http://alexcrichton.com/curl-rust)

## Quick Start

```rust
extern crate curl;

use curl::easy::Easy;

// Print a web page onto stdout
fn main() {
    let mut easy = Easy::new();
    easy.url("https://www.rust-lang.org/").unwrap();
    easy.perform().unwrap();

    println!("{}", easy.response_code().unwrap());
}
```

```rust
extern crate curl;

use curl::easy::Easy;

// Capture output into a local `Vec`.
fn main() {
    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url("https://www.rust-lang.org/").unwrap();
    easy.write_function(|data| {
        dst.extend_from_slice(data);
        data.len()
    }).unwrap();
    easy.perform().unwrap();
}
```

## Post / Put requests

Both of these methods expect that a request body is provided. A request
body can be a `&[u8]`, `&str`, or `&Reader`. For example:

```rust,no_run
extern crate curl;

use std::io::Read;
use curl::easy::Easy;

fn main() {
    let mut data = "this is the body".as_bytes();

    let mut easy = Easy::new();
    easy.url("http://www.example.com/upload").unwrap();
    easy.read_function(|buf| {
        data.read(buf).unwrap_or(0)
    }).unwrap();
    easy.post(true).unwrap();
    easy.perform().unwrap();
}
```

## Custom headers

Custom headers can be specified as part of the request:

```rust,no_run
extern crate curl;

use curl::easy::{Easy, List};

fn main() {
    let mut easy = Easy::new();
    easy.url("http://www.example.com").unwrap();

    let mut list = List::new();
    list.append("Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==").unwrap();
    easy.http_headers(list).unwrap();
    easy.perform().unwrap();
}
```

## Keep alive

The handle can be re-used across multiple requests. Curl will attempt to
keep the connections alive.

```rust,no_run
extern crate curl;

use curl::easy::Easy;

fn main() {
    let mut handle = Easy::new();

    handle.url("http://www.example.com/foo").unwrap();
    handle.perform().unwrap();

    handle.url("http://www.example.com/bar").unwrap();
    handle.perform().unwrap();
}
```

## Version Support

The bindings have been developed using curl version 7.24.0. They should
work with any newer version of curl and possibly with older versions,
but this has not been tested.

## License

The `curl-rust` crate is licensed under the MIT license, see `LICENSE` for more
details.
