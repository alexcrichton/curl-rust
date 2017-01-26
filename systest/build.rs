extern crate ctest;

use std::env;

fn main() {
    let mut cfg = ctest::TestGenerator::new();

    if let Ok(out) = env::var("DEP_CURL_INCLUDE") {
        cfg.include(&out);
    }

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

    // Disable HTTP/2 checking if feature not enabled
    #[cfg(not(feature = "http2"))]
    cfg.skip_const(|s| s.starts_with("CURL_HTTP_VERSION_2"));
    cfg.generate("../curl-sys/lib.rs", "all.rs");
}
