# Curl-Rust

libcurl bindings for rust

## Example

```rust
extern crate curl;

pub fn main() {
  let resp = curl::handle()
    .get("http://www.example.com")
    .exec();

  println!("code={}; body={}" resp.get_code(), resp.get_body());
}
```

## TODO

* Lowercase header names
* Re-use handle (for keep-alive)
