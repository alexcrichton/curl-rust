use std::env;
use std::str;

fn main() {
    let mut cfg = ctest2::TestGenerator::new();

    let mut build = cc::Build::new();
    build.file("version_detect.c");
    if let Ok(out) = env::var("DEP_CURL_INCLUDE") {
        cfg.include(&out);
        build.include(&out);
    }
    let version = build.expand();
    let version = str::from_utf8(&version).unwrap();
    let version = version
        .lines()
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(10000);
    println!("got version: {}", version);

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
    cfg.type_name(|s, is_struct, _is_union| match s {
        "CURL" | "CURLM" | "CURLSH" | "curl_version_info_data" => s.to_string(),
        "curl_khtype" | "curl_khstat" | "curl_khmatch" => format!("enum {}", s),
        s if is_struct => format!("struct {}", s),
        "sockaddr" => "struct sockaddr".to_string(),
        s => s.to_string(),
    });
    // cfg.fn_cname(|s, l| l.unwrap_or(s).to_string());
    cfg.skip_type(|n| n == "__enum_ty");
    cfg.skip_signededness(|s| s.ends_with("callback") || s.ends_with("function"));

    cfg.skip_struct(move |s| {
        if version < 71 {
            match s {
                "curl_blob" => return true,
                _ => {}
            }
        }
        if version < 70 {
            match s {
                "curl_version_info_data" => return true,
                _ => {}
            }
        }

        false
    });

    cfg.skip_const(move |s| {
        if version < 77 {
            match s {
                "CURLVERSION_TENTH"
                | "CURLOPT_CAINFO_BLOB"
                | "CURLVERSION_NOW"
                | "CURL_VERSION_ALTSVC"
                | "CURL_VERSION_ZSTD"
                | "CURL_VERSION_UNICODE"
                | "CURL_VERSION_HSTS"
                | "CURL_VERSION_GSASL" => return true,
                _ => {}
            }
        }
        if version < 75 {
            match s {
                "CURLAUTH_AWS_SIGV4" => return true,
                "CURLOPT_AWS_SIGV4" => return true,
                "CURLVERSION_NINTH" => return true,
                _ => {}
            }
        }
        if version < 72 {
            match s {
                "CURLVERSION_EIGHTH" => return true,
                _ => {}
            }
        }
        if version < 71 {
            match s {
                "CURLOPT_SSLCERT_BLOB"
                | "CURLOPT_SSLKEY_BLOB"
                | "CURLOPT_PROXY_SSLCERT_BLOB"
                | "CURLOPT_PROXY_SSLKEY_BLOB"
                | "CURLOPT_ISSUERCERT_BLOB"
                | "CURLOPTTYPE_BLOB"
                | "CURL_BLOB_NOCOPY"
                | "CURL_BLOB_COPY" => return true,
                _ => {}
            }
        }
        if version < 70 {
            match s {
                "CURL_VERSION_HTTP3" | "CURL_VERSION_BROTLI" | "CURLVERSION_SEVENTH" => {
                    return true
                }
                _ => {}
            }
        }
        if version < 66 {
            match s {
                "CURL_HTTP_VERSION_3" => return true,
                "CURLOPT_MAXAGE_CONN" => return true,
                _ => {}
            }
        }
        if version < 65 {
            match s {
                "CURLVERSION_SIXTH" => return true,
                _ => {}
            }
        }
        if version < 64 {
            match s {
                "CURLE_HTTP2" => return true,
                "CURLE_PEER_FAILED_VERIFICATION" => return true,
                "CURLE_NO_CONNECTION_AVAILABLE" => return true,
                "CURLE_SSL_PINNEDPUBKEYNOTMATCH" => return true,
                "CURLE_SSL_INVALIDCERTSTATUS" => return true,
                "CURLE_HTTP2_STREAM" => return true,
                "CURLE_RECURSIVE_API_CALL" => return true,
                _ => {}
            }
        }
        if version < 62 {
            match s {
                "CURLOPT_UPLOAD_BUFFERSIZE" => return true,
                _ => {}
            }
        }
        if version < 61 {
            match s {
                "CURLOPT_PIPEWAIT" => return true,
                "CURLE_PEER_FAILED_VERIFICATION" => return true,
                _ => {}
            }
        }
        if version < 60 {
            match s {
                "CURLVERSION_FIFTH" => return true,
                _ => {}
            }
        }
        if version < 54 {
            match s {
                "CURL_SSLVERSION_TLSv1_3"
                | "CURLOPT_PROXY_CAINFO"
                | "CURLOPT_PROXY_SSLCERT"
                | "CURLOPT_PROXY_SSLKEY" => return true,
                _ => {}
            }
        }
        if version < 52 {
            match s {
                "CURLOPT_PROXY_CAPATH" => return true,
                _ => {}
            }
        }

        if version < 49 {
            match s {
                "CURL_HTTP_VERSION_2_PRIOR_KNOWLEDGE" | "CURLOPT_CONNECT_TO" => return true,
                _ => {}
            }
        }

        if version < 47 {
            if s.starts_with("CURL_HTTP_VERSION_2") {
                return true;
            }
        }

        if version < 43 {
            if s.starts_with("CURLPIPE_") {
                return true;
            }
        }

        // OSX doesn't have this yet
        s == "CURLSSLOPT_NO_REVOKE" ||

        // A lot of curl versions doesn't support unix sockets
        s == "CURLOPT_UNIX_SOCKET_PATH" || s == "CURL_VERSION_UNIX_SOCKETS"
    });

    if cfg!(target_env = "msvc") {
        cfg.skip_fn_ptrcheck(|s| s.starts_with("curl_"));
    }

    cfg.generate("../curl-sys/lib.rs", "all.rs");
}
