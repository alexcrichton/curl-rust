//! Simple HTTP/3 GET
//!
//! This example is a Rust adaptation of the [C example of the same
//! name](https://curl.se/libcurl/c/http3.html).

use curl::easy::Easy;

fn main() -> Result<(), curl::Error> {
    let mut curl = Easy::new();

    // An HTTP/3-capable server.
    curl.url("https://cloudflare-quic.com")?;

    // Force HTTP/3 to be used.
    curl.http_version(curl::easy::HttpVersion::V3)?;

    curl.perform()?;

    Ok(())
}
