# curl-rust

[libcurl] bindings for Rust

[![Latest Version](https://img.shields.io/crates/v/curl.svg)](https://crates.io/crates/curl)
[![Documentation](https://docs.rs/curl/badge.svg)](https://docs.rs/curl)
[![License](https://img.shields.io/github/license/alexcrichton/curl-rust.svg)](LICENSE)
[![Build](https://github.com/alexcrichton/curl-rust/workflows/CI/badge.svg)](https://github.com/alexcrichton/curl-rust/actions)

## Quick Start

```rust
use std::io::{stdout, Write};

use curl::easy::Easy;

// Print a web page onto stdout
fn main() {
    let mut easy = Easy::new();
    easy.url("https://www.rust-lang.org/").unwrap();
    easy.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    easy.perform().unwrap();

    println!("{}", easy.response_code().unwrap());
}
```

```rust
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

## Building

By default, this crate will attempt to dynamically link to the system-wide
libcurl and the system-wide SSL library. Some of this behavior can be customized
with various Cargo features:

- `ssl`: Enable SSL/TLS support using the platform-default TLS backend. On Windows this is [Schannel], on macOS [Secure Transport], and [OpenSSL] (or equivalent) on all other platforms.  Enabled by default.
- `mesalink`: Enable SSL/TLS support via [MesaLink], an alternative TLS backend written in Rust based on [Rustls]. MesaLink is always statically linked. Disabled by default.
- `http2`: Enable HTTP/2 support via libnghttp2. Disabled by default.
- `static-curl`: Use a bundled libcurl version and statically link to it. Disabled by default.
- `static-ssl`: Use a bundled OpenSSL version and statically link to it. Only applies on platforms that use OpenSSL. Disabled by default.
- `spnego`: Enable SPNEGO support. Disabled by default.

## Version Support

The bindings have been developed using curl version 7.24.0. They should
work with any newer version of curl and possibly with older versions,
but this has not been tested.

## Troubleshooting

### Curl built against the NSS SSL library

If you encounter the following error message:

```
  [77] Problem with the SSL CA cert (path? access rights?)
```

That means most likely, that curl was linked against `libcurl-nss.so` due to
installed libcurl NSS development files, and that the required library
`libnsspem.so` is missing. See also the curl man page: "If curl is built
against the NSS SSL library, the NSS PEM PKCS#11 module (`libnsspem.so`) needs to be available for this option to work properly."

In order to avoid this failure you can either

 * install the missing library (e.g. Debian: `nss-plugin-pem`), or
 * remove the libcurl NSS development files (e.g. Debian: `libcurl4-nss-dev`) and
   rebuild curl-rust.

## License

The `curl-rust` crate is licensed under the MIT license, see [`LICENSE`](LICENSE) for more
details.


[libcurl]: https://curl.haxx.se/libcurl/
[MesaLink]: https://mesalink.io/
[OpenSSL]: https://www.openssl.org/
[Rustls]: https://github.com/ctz/rustls
[Schannel]: https://docs.microsoft.com/en-us/windows/win32/com/schannel
[Secure Transport]: https://developer.apple.com/documentation/security/secure_transport
