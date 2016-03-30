extern crate ctest;

use std::env;

fn main() {
    let mut cfg = ctest::TestGenerator::new();

    cfg.header("curl/curl.h");
    if let Ok(out) = env::var("DEP_CURL_INCLUDE") {
        cfg.include(&out);
    }
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
    cfg.generate("../curl-sys/lib.rs", "all.rs");
}
