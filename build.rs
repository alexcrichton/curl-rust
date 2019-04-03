use std::env;
use std::str::FromStr;

fn main() {
    // OpenSSL >= 1.1.0 can be initialized concurrently and is initialized correctly by libcurl.
    // <= 1.0.2 need locking callbacks, which are provided by openssl_sys::init().
    let use_openssl = match env::var("DEP_OPENSSL_VERSION") {
        Ok(ver) => {
            let ver = u32::from_str(&ver).unwrap();
            if ver < 110 {
                println!("cargo:rustc-cfg=need_openssl_init");
            }
            true
        }
        Err(_) => false,
    };

    if use_openssl {
        // The system libcurl should have the default certificate paths configured.
        if env::var_os("DEP_CURL_STATIC").is_some() {
            println!("cargo:rustc-cfg=need_openssl_probe");
        }
    }
}
