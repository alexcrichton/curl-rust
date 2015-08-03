# Curl-Rust

libcurl bindings for Rust. **This project is in active development and
could (will probably) break API compatibility at any time**.

[![Build Status](https://travis-ci.org/carllerche/curl-rust.svg?branch=master)](https://travis-ci.org/carllerche/curl-rust)
[![crates.io](http://meritbadge.herokuapp.com/mio)](https://crates.io/crates/curl)

## Quick Start

```rust
extern crate curl;

use curl::http;

pub fn main() {
  let resp = http::handle()
    .get("http://www.example.com")
    .exec().unwrap();

  println!("code={}; headers={:?}; body={:?}",
    resp.get_code(), resp.get_headers(), resp.get_body());
}
```

Response header names are automatically lower cased.

## Post / Put requests

Both of these methods expect that a request body is provided. A request
body can be a `&[u8]`, `&str`, or `&Reader`. For example:

```rust
let resp = http::handle()
  .post("http://www.example.com/upload", "this is the body")
  .exec().unwrap();
```

## Custom headers

Custom headers can be specified as part of the request:

```rust
http::handle()
  .get("http://www.example.com")
  .header("Authorization", "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==")
  .exec();
```

## Keep alive

The handle can be re-used across multiple requests. Curl will attempt to
keep the connections alive.

```rust
let handle = http::handle();

let resp1 = handle.get("http://www.example.com/foo").exec().unwrap();
let resp2 = handle.get("http://www.example.com/bar").exec().unwrap();
```

## Version Support

The bindings have been developed using curl version 7.24.0. They should
work with any newer version of curl and possibly with older versions,
but this has not been tested.
