# Curl-Rust

libcurl bindings for rust

## Example

```rust
extern crate curl;

pub fn main() {
  let resp = curl::handle()
    .get("http://www.example.com")
    .exec().unwrap();

  println!("code={}; body={}" resp.get_code(), resp.get_body());
}
```

## TODO

* Lowercase header names
* Re-use handle (for keep-alive)
* Response body should be optional
* Support SSL
* Support proxies
* Keep-alive configuration
* Whether or not to follow redirects (FOLLOWLOCATION, MAXREDIRS)
