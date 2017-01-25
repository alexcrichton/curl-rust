# curl-rust

libcurl bindings for Rust

[![Build Status](https://travis-ci.org/alexcrichton/curl-rust.svg?branch=master)](https://travis-ci.org/alexcrichton/curl-rust)
[![Build status](https://ci.appveyor.com/api/projects/status/lx98wtbxhhhajpr9?svg=true)](https://ci.appveyor.com/project/alexcrichton/curl-rust)

[Documentation](https://docs.rs/curl)

## Quick Start

```rust
extern crate curl;

use std::io::{stdout, Write};

use curl::easy::Easy;

// Print a web page onto stdout
fn main() {
    let mut easy = Easy::new();
    easy.url("https://www.rust-lang.org/").unwrap();
    easy.write_function(|data| {
        Ok(stdout().write(data).unwrap())
    }).unwrap();
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

    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        dst.extend_from_slice(data);
        Ok(data.len())
    }).unwrap();
    transfer.perform().unwrap();
}
```

## Post / Put requests

The `put` and `post` methods on `Easy` can configure the method of the HTTP
request, and then `read_function` can be used to specify how data is filled in.
This interface works particularly well with types that implement `Read`.

```rust,no_run
extern crate curl;

use std::io::Read;
use curl::easy::Easy;

fn main() {
    let mut data = "this is the body".as_bytes();

    let mut easy = Easy::new();
    easy.url("http://www.example.com/upload").unwrap();
    easy.post(true).unwrap();
    easy.post_field_size(data.len() as u64).unwrap();

    let mut transfer = easy.transfer();
    transfer.read_function(|buf| {
        Ok(data.read(buf).unwrap_or(0))
    }).unwrap();
    transfer.perform().unwrap();
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

## Multiple requests

The libcurl library provides support for sending multiple requests
simultaneously through the "multi" interface. This is currently bound in the
`multi` module of this crate and provides the ability to execute multiple
transfers simultaneously. For more information, see that module.

## Version Support

The bindings have been developed using curl version 7.24.0. They should
work with any newer version of curl and possibly with older versions,
but this has not been tested.

## License

The `curl-rust` crate is licensed under the MIT license, see `LICENSE` for more
details.
