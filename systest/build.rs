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

    cfg.skip_const(|s| {
        // Ubuntu Xenial 16.04 ships with version 7.47.0 of curl, so explicitly
        // skip constants introduced after that.
        // CURL_HTTP_VERSION_2_PRIOR_KNOWLEDGE was introduced in 7.49.0
        s == "CURL_HTTP_VERSION_2_PRIOR_KNOWLEDGE" ||

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
