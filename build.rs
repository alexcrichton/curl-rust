use std::env;
use std::str::FromStr;

fn main() {
    let use_openssl;

    // OpenSSL >= 1.1.0 can be initialized concurrently and is initialized correctly by libcurl.
    // <= 1.0.2 need locking callbacks, which are provided by openssl_sys::init().
    match env::var("DEP_OPENSSL_VERSION") {
        Ok(ver) => {
            use_openssl = true;
            let ver = u32::from_str(&ver).unwrap();
            if ver < 110 {
                println!("cargo:rustc-cfg=need_openssl_init");
            }

        }
        Err(env::VarError::NotPresent) => {
            use_openssl = false;
        }
        Err(e) => panic!(e),
    }

    if use_openssl {
        // The system libcurl should have the default certificate paths configured.
        match env::var_os("DEP_CURL_STATIC") {
            Some(_) => {
                println!("cargo:rustc-cfg=need_openssl_probe");
            }
            None => {}
        }
    }
}
