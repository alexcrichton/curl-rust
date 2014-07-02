# Curl-Rust

libcurl bindings for rust

## Example

```rust
extern crate curl;

use curl::http;

pub fn main() {
  let resp = http::handle()
    .get("http://www.example.com")
    .exec().unwrap();

  println!("code={}; body={}" resp.get_code(), resp.get_body());
}
```
