use std::env;
use std::str::FromStr;

fn main() {
    // OpenSSL >= 1.1.0 can be initialized concurrently and is initialized correctly by libcurl.
    // <= 1.0.2 need locking callbacks, which are provided by openssl_sys::init().
    match env::var("DEP_OPENSSL_VERSION") {
        Ok(ver) => {
            let ver = u32::from_str(&ver).unwrap();
            if ver < 110 {
                println!("cargo:rustc-cfg=need_openssl_init");
            }
        }
        Err(env::VarError::NotPresent) => {}
        Err(e) => panic!(e),
    }

    // The system libcurl should have the default certificate paths configured.
    match env::var("DEP_CURL_STATIC") {
        Ok(_) => {
            println!("cargo:rustc-cfg=need_openssl_probe");
        }
        Err(env::VarError::NotPresent) => {}
        Err(e) => panic!(e),
    }
}
