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
        .and_then(|s| {
            let mut parts = s.split_whitespace();
            let major = parts.next()?.parse::<u32>().ok()?;
            let minor = parts.next()?.parse::<u32>().ok()?;
            Some((major, minor))
        })
        .unwrap_or((10000, 0));
    println!("got version: {version:?}");

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
        "__enum_ty" => "unsigned".to_string(),
        s => s.to_string(),
    });
    // cfg.fn_cname(|s, l| l.unwrap_or(s).to_string());
    cfg.skip_type(|n| n == "__enum_ty");
    cfg.skip_signededness(|s| s.ends_with("callback") || s.ends_with("function"));

    cfg.skip_struct(move |s| {
        if version < (7, 71) {
            match s {
                "curl_blob" => return true,
                _ => {}
            }
        }
        if version < (8, 10) {
            match s {
                "curl_version_info_data" => return true,
                _ => {}
            }
        }

        false
    });

    // Version symbols are extracted from https://curl.se/libcurl/c/symbols-in-versions.html
    cfg.skip_const(move |s| {
        if version < (8, 10) {
            match s {
                "CURLVERSION_TWELFTH" | "CURLVERSION_NOW" => return true,
                _ => {}
            }
        }
        if version < (7, 87) {
            match s {
                "CURLVERSION_ELEVENTH" => return true,
                _ => {}
            }
        }
        if version < (7, 77) {
            match s {
                "CURLVERSION_TENTH"
                | "CURLOPT_CAINFO_BLOB"
                | "CURLOPT_PROXY_CAINFO_BLOB"
                | "CURL_VERSION_ALTSVC"
                | "CURL_VERSION_ZSTD"
                | "CURL_VERSION_UNICODE"
                | "CURL_VERSION_HSTS"
                | "CURL_VERSION_GSASL"
                | "CURLSSLOPT_AUTO_CLIENT_CERT" => return true,
                _ => {}
            }
        }
        if version < (7, 76) {
            match s {
                "CURLOPT_DOH_SSL_VERIFYHOST" => return true,
                "CURLOPT_DOH_SSL_VERIFYPEER" => return true,
                "CURLOPT_DOH_SSL_VERIFYSTATUS" => return true,
                _ => {}
            }
        }
        if version < (7, 75) {
            match s {
                "CURLAUTH_AWS_SIGV4" => return true,
                "CURLOPT_AWS_SIGV4" => return true,
                "CURLVERSION_NINTH" => return true,
                _ => {}
            }
        }
        if version < (7, 72) {
            match s {
                "CURLVERSION_EIGHTH" => return true,
                _ => {}
            }
        }
        if version < (7, 71) {
            match s {
                "CURLOPT_SSLCERT_BLOB"
                | "CURLOPT_SSLKEY_BLOB"
                | "CURLOPT_PROXY_ISSUERCERT_BLOB"
                | "CURLOPT_PROXY_ISSUERCERT"
                | "CURLOPT_PROXY_SSLCERT_BLOB"
                | "CURLOPT_PROXY_SSLKEY_BLOB"
                | "CURLOPT_ISSUERCERT_BLOB"
                | "CURLOPTTYPE_BLOB"
                | "CURL_BLOB_NOCOPY"
                | "CURL_BLOB_COPY"
                | "CURLSSLOPT_NATIVE_CA" => return true,
                _ => {}
            }
        }
        if version < (7, 70) {
            match s {
                "CURL_VERSION_HTTP3"
                | "CURL_VERSION_BROTLI"
                | "CURLVERSION_SEVENTH"
                | "CURLSSLOPT_REVOKE_BEST_EFFORT" => return true,
                _ => {}
            }
        }
        if version < (7, 68) {
            match s {
                "CURLSSLOPT_NO_PARTIALCHAIN" => return true,
                _ => {}
            }
        }
        if version < (7, 67) {
            match s {
                "CURLMOPT_MAX_CONCURRENT_STREAMS" => return true,
                _ => {}
            }
        }
        if version < (7, 66) {
            match s {
                "CURL_HTTP_VERSION_3" => return true,
                "CURLOPT_MAXAGE_CONN" => return true,
                _ => {}
            }
        }
        if version < (7, 65) {
            match s {
                "CURLVERSION_SIXTH" => return true,
                _ => {}
            }
        }
        if version < (7, 64) {
            match s {
                "CURLE_HTTP2" => return true,
                "CURLE_PEER_FAILED_VERIFICATION" => return true,
                "CURLE_NO_CONNECTION_AVAILABLE" => return true,
                "CURLE_SSL_PINNEDPUBKEYNOTMATCH" => return true,
                "CURLE_SSL_INVALIDCERTSTATUS" => return true,
                "CURLE_HTTP2_STREAM" => return true,
                "CURLE_RECURSIVE_API_CALL" => return true,
                "CURLOPT_HTTP09_ALLOWED" => return true,
                _ => {}
            }
        }
        if version < (7, 62) {
            match s {
                "CURLOPT_DOH_URL" => return true,
                "CURLOPT_UPLOAD_BUFFERSIZE" => return true,
                _ => {}
            }
        }
        if version < (7, 61) {
            match s {
                "CURLOPT_PIPEWAIT" => return true,
                "CURLE_PEER_FAILED_VERIFICATION" => return true,
                _ => {}
            }
        }
        if version < (7, 60) {
            match s {
                "CURLVERSION_FIFTH" => return true,
                _ => {}
            }
        }
        if version < (7, 54) {
            match s {
                "CURL_SSLVERSION_TLSv1_3" | "CURLOPT_PROXY_SSLCERT" | "CURLOPT_PROXY_SSLKEY" => {
                    return true
                }
                _ => {}
            }
        }
        if version < (7, 52) {
            match s {
                "CURLOPT_PROXY_CAINFO"
                | "CURLOPT_PROXY_CAPATH"
                | "CURLOPT_PROXY_CRLFILE"
                | "CURLOPT_PROXY_KEYPASSWD"
                | "CURLOPT_PROXY_SSL_CIPHER_LIST"
                | "CURLOPT_PROXY_SSL_OPTIONS"
                | "CURLOPT_PROXY_SSL_VERIFYHOST"
                | "CURLOPT_PROXY_SSL_VERIFYPEER"
                | "CURLOPT_PROXY_SSLCERT"
                | "CURLOPT_PROXY_SSLCERTTYPE"
                | "CURLOPT_PROXY_SSLKEY"
                | "CURLOPT_PROXY_SSLKEYTYPE"
                | "CURLOPT_PROXY_SSLVERSION"
                | "CURL_VERSION_HTTPS_PROXY" => return true,
                _ => {}
            }
        }

        if version < (7, 49) {
            match s {
                "CURL_HTTP_VERSION_2_PRIOR_KNOWLEDGE" | "CURLOPT_CONNECT_TO" => return true,
                _ => {}
            }
        }
        if version < (7, 47) {
            if s.starts_with("CURL_HTTP_VERSION_2") {
                return true;
            }
        }
        if version < (7, 44) {
            match s {
                "CURLMOPT_PUSHDATA" | "CURLMOPT_PUSHFUNCTION" => return true,
                _ => {}
            }
        }
        if version < (7, 43) {
            if s.starts_with("CURLPIPE_") {
                return true;
            }
        }
        if version < (7, 25) {
            match s {
                "CURLSSLOPT_ALLOW_BEAST" => return true,
                _ => {}
            }
        }

        match s {
            // OSX doesn't have this yet
            "CURLSSLOPT_NO_REVOKE"
            // A lot of curl versions doesn't support unix sockets
            | "CURLOPT_UNIX_SOCKET_PATH"
            | "CURL_VERSION_UNIX_SOCKETS"
            | "CURLOPT_ABSTRACT_UNIX_SOCKET"
            // These two are deprecated, and their value changed in 8.10.
            // Systest generates deprecated warnings which isn't helpful.
            // These should be removed in the next semver major bump.
            | "CURLOPT_WRITEINFO"
            | "CURLOPT_CLOSEPOLICY" => true,
            _ => false,
        }
    });

    if cfg!(target_env = "msvc") {
        cfg.skip_fn_ptrcheck(|s| s.starts_with("curl_"));
    }

    cfg.generate("../curl-sys/lib.rs", "all.rs");
}
