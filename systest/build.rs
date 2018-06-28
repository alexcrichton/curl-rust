extern crate ctest;
extern crate cc;

use std::env;
use std::str;

fn main() {
    let mut cfg = ctest::TestGenerator::new();

    let mut build = cc::Build::new();
    build.file("version_detect.c");
    if let Ok(out) = env::var("DEP_CURL_INCLUDE") {
        cfg.include(&out);
        build.include(&out);
    }
    let version = build.expand();
    let version = str::from_utf8(&version).unwrap();
    let version = version.lines()
        .filter(|l| !l.is_empty() && !l.starts_with("#"))
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(10000);

    if env::var("TARGET").unwrap().contains("msvc") {
        cfg.flag("/wd4574"); // did you mean to use '#if INCL_WINSOCK_API_TYPEDEFS'
    }

    cfg.header("curl/curl.h");
    cfg.define("CURL_STATICLIB", None);
    cfg.field_name(|s, field| {
        if s == "curl_fileinfo" {
            field.replace("strings_", "strings.")
        } else if s == "CURLMsg" && field == "data" {
            "data.whatever".to_string()
        } else {
            field.to_string()
        }
    });
    cfg.type_name(|s, is_struct| {
        match s {
            "CURL" |
            "CURLM" |
            "CURLSH" |
            "curl_version_info_data" => s.to_string(),
            "curl_khtype" |
            "curl_khstat" |
            "curl_khmatch" => format!("enum {}", s),
            s if is_struct => format!("struct {}", s),
            "sockaddr" => format!("struct sockaddr"),
            s => s.to_string(),
        }
    });
    // cfg.fn_cname(|s, l| l.unwrap_or(s).to_string());
    cfg.skip_type(|n| n == "__enum_ty");
    cfg.skip_signededness(|s| {
        s.ends_with("callback") || s.ends_with("function")
    });

    cfg.skip_struct(move |s| {
        if version < 60 {
            match s {
                "curl_version_info_data" => return true,
                _ => {}
            }
        }

        false
    });

    cfg.skip_const(move |s| {
        if version < 60 {
            match s {
                "CURLVERSION_FIFTH" |
                "CURLVERSION_NOW" => return true,
                _ => {}
            }
        }

        if version < 49 {
            if s.starts_with("CURL_HTTP_VERSION_2_PRIOR_KNOWLEDGE") {
                return true
            }
        }

        if version < 43 {
            if s.starts_with("CURLPIPE_") {
                return true
            }
        }

        // OSX doesn't have this yet
        s == "CURLSSLOPT_NO_REVOKE" ||

        // Disable HTTP/2 checking if feature not enabled
        (!cfg!(feature = "http2") && s.starts_with("CURL_HTTP_VERSION_2")) ||

        // A lot of curl versions doesn't support unix sockets
        s == "CURLOPT_UNIX_SOCKET_PATH" || s == "CURL_VERSION_UNIX_SOCKETS"
    });

    if cfg!(target_env = "msvc") {
        cfg.skip_fn_ptrcheck(|s| s.starts_with("curl_"));
    }

    cfg.generate("../curl-sys/lib.rs", "all.rs");
}
