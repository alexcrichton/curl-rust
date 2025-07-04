use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=curl");
    println!(
        "cargo:rustc-check-cfg=cfg(\
            libcurl_vendored,\
            link_libnghttp2,\
            link_libz,\
            link_openssl,\
        )"
    );
    let target = env::var("TARGET").unwrap();
    let windows = target.contains("windows");

    if cfg!(feature = "mesalink") {
        println!("cargo:warning=MesaLink support has been removed as of curl 7.82.0, will use default TLS backend instead.");
    }

    // This feature trumps all others, and is largely set by rustbuild to force
    // usage of the system library to ensure that we're always building an
    // ABI-compatible Cargo.
    if cfg!(feature = "force-system-lib-on-osx") && target.contains("apple") {
        return println!("cargo:rustc-flags=-l curl");
    }

    // If the static-curl feature is disabled, probe for a system-wide libcurl.
    if !cfg!(feature = "static-curl") {
        // OSX ships libcurl by default, so we just use that version
        // so long as it has the right features enabled.
        if target.contains("apple") && (!cfg!(feature = "http2") || curl_config_reports_http2()) {
            return println!("cargo:rustc-flags=-l curl");
        }

        // Next, fall back and try to use pkg-config if its available.
        if windows {
            if try_vcpkg() {
                return;
            }
        } else if try_pkg_config() {
            return;
        }
    }

    if !Path::new("curl/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init", "curl"])
            .status();
    }

    if target.contains("apple") {
        // On (older) OSX we need to link against the clang runtime,
        // which is hidden in some non-default path.
        //
        // More details at https://github.com/alexcrichton/curl-rust/issues/279.
        if let Some(path) = macos_link_search_path() {
            println!("cargo:rustc-link-lib=clang_rt.osx");
            println!("cargo:rustc-link-search={}", path);
        }
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let include = dst.join("include");
    let build = dst.join("build");
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}", include.display());
    println!("cargo:static=1");
    println!("cargo:rustc-cfg=libcurl_vendored");
    fs::create_dir_all(include.join("curl")).unwrap();

    for header in [
        "curl.h",
        "curlver.h",
        "easy.h",
        "options.h",
        "header.h",
        "mprintf.h",
        "multi.h",
        "stdcheaders.h",
        "system.h",
        "urlapi.h",
        "typecheck-gcc.h",
        "websockets.h",
    ]
    .iter()
    {
        fs::copy(
            format!("curl/include/curl/{}", header),
            include.join("curl").join(header),
        )
        .unwrap();
    }

    let pkgconfig = dst.join("lib/pkgconfig");
    fs::create_dir_all(&pkgconfig).unwrap();
    let contents = fs::read_to_string("curl/libcurl.pc.in").unwrap();
    fs::write(
        pkgconfig.join("libcurl.pc"),
        contents
            .replace("@prefix@", dst.to_str().unwrap())
            .replace("@exec_prefix@", "")
            .replace("@libdir@", dst.join("lib").to_str().unwrap())
            .replace("@includedir@", include.to_str().unwrap())
            .replace("@CPPFLAG_CURL_STATICLIB@", "-DCURL_STATICLIB")
            .replace("@LIBCURL_LIBS@", "")
            .replace("@SUPPORT_FEATURES@", "")
            .replace("@SUPPORT_PROTOCOLS@", "")
            .replace("@CURLVERSION@", "8.14.1"),
    )
    .unwrap();

    let mut cfg = cc::Build::new();
    cfg.out_dir(&build)
        .include("curl/lib")
        .include("curl/include")
        .define("BUILDING_LIBCURL", None)
        .define("CURL_DISABLE_DICT", None)
        .define("CURL_DISABLE_GOPHER", None)
        .define("CURL_DISABLE_IMAP", None)
        .define("CURL_DISABLE_LDAP", None)
        .define("CURL_DISABLE_LDAPS", None)
        .define("CURL_DISABLE_POP3", None)
        .define("CURL_DISABLE_RTSP", None)
        .define("CURL_DISABLE_SMB", None)
        .define("CURL_DISABLE_SMTP", None)
        .define("CURL_DISABLE_TELNET", None)
        .define("CURL_DISABLE_TFTP", None)
        .define("CURL_STATICLIB", None)
        .define("ENABLE_IPV6", None)
        .define("HAVE_SOCKADDR_IN6_SIN6_SCOPE_ID", None)
        .define("HAVE_ASSERT_H", None)
        .define("CURL_OS", "\"unknown\"") // TODO
        .define("HAVE_ZLIB_H", None)
        .define("HAVE_LONGLONG", None)
        .define("HAVE_LIBZ", None)
        .define("HAVE_BOOL_T", None)
        .define("HAVE_STDBOOL_H", None)
        .file("curl/lib/altsvc.c")
        .file("curl/lib/asyn-base.c")
        .file("curl/lib/asyn-thrdd.c")
        .file("curl/lib/bufq.c")
        .file("curl/lib/bufref.c")
        .file("curl/lib/cf-h1-proxy.c")
        .file("curl/lib/cf-haproxy.c")
        .file("curl/lib/cf-https-connect.c")
        .file("curl/lib/cf-socket.c")
        .file("curl/lib/cfilters.c")
        .file("curl/lib/conncache.c")
        .file("curl/lib/connect.c")
        .file("curl/lib/content_encoding.c")
        .file("curl/lib/cookie.c")
        .file("curl/lib/cshutdn.c")
        .file("curl/lib/curl_addrinfo.c")
        .file("curl/lib/curl_get_line.c")
        .file("curl/lib/curl_memrchr.c")
        .file("curl/lib/curl_range.c")
        .file("curl/lib/curl_sha512_256.c")
        .file("curl/lib/curl_threads.c")
        .file("curl/lib/curl_trc.c")
        .file("curl/lib/curlx/base64.c")
        .file("curl/lib/curlx/dynbuf.c")
        .file("curl/lib/curlx/inet_pton.c")
        .file("curl/lib/curlx/nonblock.c")
        .file("curl/lib/curlx/strparse.c")
        .file("curl/lib/curlx/timediff.c")
        .file("curl/lib/curlx/timeval.c")
        .file("curl/lib/curlx/warnless.c")
        .file("curl/lib/cw-out.c")
        .file("curl/lib/cw-pause.c")
        .file("curl/lib/doh.c")
        .file("curl/lib/dynhds.c")
        .file("curl/lib/easy.c")
        .file("curl/lib/escape.c")
        .file("curl/lib/file.c")
        .file("curl/lib/fileinfo.c")
        .file("curl/lib/fopen.c")
        .file("curl/lib/formdata.c")
        .file("curl/lib/getenv.c")
        .file("curl/lib/getinfo.c")
        .file("curl/lib/hash.c")
        .file("curl/lib/headers.c")
        .file("curl/lib/hmac.c")
        .file("curl/lib/hostip.c")
        .file("curl/lib/hostip6.c")
        .file("curl/lib/hsts.c")
        .file("curl/lib/http.c")
        .file("curl/lib/http1.c")
        .file("curl/lib/http_aws_sigv4.c")
        .file("curl/lib/http_chunks.c")
        .file("curl/lib/http_digest.c")
        .file("curl/lib/http_proxy.c")
        .file("curl/lib/idn.c")
        .file("curl/lib/if2ip.c")
        .file("curl/lib/inet_ntop.c")
        .file("curl/lib/llist.c")
        .file("curl/lib/macos.c")
        .file("curl/lib/md5.c")
        .file("curl/lib/mime.c")
        .file("curl/lib/mprintf.c")
        .file("curl/lib/mqtt.c")
        .file("curl/lib/multi.c")
        .file("curl/lib/multi_ev.c")
        .file("curl/lib/netrc.c")
        .file("curl/lib/noproxy.c")
        .file("curl/lib/parsedate.c")
        .file("curl/lib/progress.c")
        .file("curl/lib/rand.c")
        .file("curl/lib/rename.c")
        .file("curl/lib/request.c")
        .file("curl/lib/select.c")
        .file("curl/lib/sendf.c")
        .file("curl/lib/setopt.c")
        .file("curl/lib/sha256.c")
        .file("curl/lib/share.c")
        .file("curl/lib/slist.c")
        .file("curl/lib/socketpair.c")
        .file("curl/lib/socks.c")
        .file("curl/lib/speedcheck.c")
        .file("curl/lib/splay.c")
        .file("curl/lib/strcase.c")
        .file("curl/lib/strdup.c")
        .file("curl/lib/strequal.c")
        .file("curl/lib/strerror.c")
        .file("curl/lib/transfer.c")
        .file("curl/lib/uint-bset.c")
        .file("curl/lib/uint-hash.c")
        .file("curl/lib/uint-spbset.c")
        .file("curl/lib/uint-table.c")
        .file("curl/lib/url.c")
        .file("curl/lib/urlapi.c")
        .file("curl/lib/vauth/digest.c")
        .file("curl/lib/vauth/vauth.c")
        .file("curl/lib/version.c")
        .file("curl/lib/vquic/curl_msh3.c")
        .file("curl/lib/vquic/curl_ngtcp2.c")
        .file("curl/lib/vquic/curl_osslq.c")
        .file("curl/lib/vquic/curl_quiche.c")
        .file("curl/lib/vquic/vquic-tls.c")
        .file("curl/lib/vquic/vquic.c")
        .file("curl/lib/vtls/hostcheck.c")
        .file("curl/lib/vtls/keylog.c")
        .file("curl/lib/vtls/vtls.c")
        .file("curl/lib/vtls/vtls_scache.c")
        .file("curl/lib/ws.c")
        .define("HAVE_GETADDRINFO", None)
        .define("HAVE_GETPEERNAME", None)
        .define("HAVE_GETSOCKNAME", None)
        .warnings(false);

    if cfg!(feature = "ntlm") {
        cfg.file("curl/lib/curl_des.c")
            .file("curl/lib/curl_endian.c")
            .file("curl/lib/curl_gethostname.c")
            .file("curl/lib/curl_ntlm_core.c")
            .file("curl/lib/http_ntlm.c")
            .file("curl/lib/md4.c")
            .file("curl/lib/vauth/ntlm.c")
            .file("curl/lib/vauth/ntlm_sspi.c");
    } else {
        cfg.define("CURL_DISABLE_NTLM", None);
    }

    if cfg!(feature = "protocol-ftp") {
        cfg.file("curl/lib/curl_fnmatch.c")
            .file("curl/lib/ftp.c")
            .file("curl/lib/ftplistparser.c")
            .file("curl/lib/pingpong.c");
    } else {
        cfg.define("CURL_DISABLE_FTP", None);
    }

    if cfg!(feature = "http2") {
        cfg.define("USE_NGHTTP2", None)
            .define("NGHTTP2_STATICLIB", None)
            .file("curl/lib/cf-h2-proxy.c")
            .file("curl/lib/http2.c");

        println!("cargo:rustc-cfg=link_libnghttp2");
        if let Some(path) = env::var_os("DEP_NGHTTP2_ROOT") {
            let path = PathBuf::from(path);
            cfg.include(path.join("include"));
        }
    }

    println!("cargo:rustc-cfg=link_libz");
    if let Some(path) = env::var_os("DEP_Z_INCLUDE") {
        cfg.include(path);
    }

    if cfg!(feature = "spnego") {
        cfg.define("USE_SPNEGO", None)
            .file("curl/lib/http_negotiate.c")
            .file("curl/lib/vauth/vauth.c");
    }

    // Configure TLS backend. Since Cargo does not support mutually exclusive
    // features, make sure we only compile one vtls.
    if cfg!(feature = "rustls") {
        cfg.define("USE_RUSTLS", None)
            .file("curl/lib/vtls/cipher_suite.c")
            .file("curl/lib/vtls/rustls.c")
            .file("curl/lib/vtls/x509asn1.c")
            .include(env::var_os("DEP_RUSTLS_FFI_INCLUDE").unwrap());
    } else if cfg!(feature = "windows-static-ssl") {
        if windows {
            cfg.define("USE_OPENSSL", None)
                .file("curl/lib/vtls/openssl.c");
            // We need both openssl and zlib
            // Those can be installed with
            // ```shell
            // git clone https://github.com/microsoft/vcpkg
            // cd vcpkg
            // ./bootstrap-vcpkg.bat -disableMetrics
            // ./vcpkg.exe integrate install
            // ./vcpkg.exe install openssl:x64-windows-static-md
            // ```
            #[cfg(target_env = "msvc")]
            vcpkg::Config::new().find_package("openssl").ok();
            #[cfg(target_env = "msvc")]
            vcpkg::Config::new().find_package("zlib").ok();
        } else {
            panic!("Not available on non windows platform")
        }
    } else if cfg!(feature = "ssl") {
        if windows {
            // For windows, spnego feature is auto on in case ssl feature is on.
            // Please see definition of USE_SPNEGO in curl_setup.h for more info.
            cfg.define("USE_WINDOWS_SSPI", None)
                .define("USE_SCHANNEL", None)
                .file("curl/lib/http_negotiate.c")
                .file("curl/lib/curl_sspi.c")
                .file("curl/lib/socks_sspi.c")
                .file("curl/lib/vauth/spnego_sspi.c")
                .file("curl/lib/vauth/vauth.c")
                .file("curl/lib/vtls/schannel.c")
                .file("curl/lib/vtls/schannel_verify.c")
                .file("curl/lib/vtls/x509asn1.c");
        } else if target.contains("-apple-") {
            cfg.define("USE_SECTRANSP", None)
                .file("curl/lib/vtls/cipher_suite.c")
                .file("curl/lib/vtls/sectransp.c")
                .file("curl/lib/vtls/x509asn1.c");
            if xcode_major_version().map_or(true, |v| v >= 9) {
                // On earlier Xcode versions (<9), defining HAVE_BUILTIN_AVAILABLE
                // would cause __bultin_available() to fail to compile due to
                // unrecognized platform names, so we try to check for Xcode
                // version first (if unknown, assume it's recent, as in >= 9).
                cfg.define("HAVE_BUILTIN_AVAILABLE", "1");
            }
        } else {
            cfg.define("USE_OPENSSL", None)
                .file("curl/lib/vtls/openssl.c");

            println!("cargo:rustc-cfg=link_openssl");
            if let Some(path) = env::var_os("DEP_OPENSSL_INCLUDE") {
                cfg.include(path);
            }
        }
    }

    // Configure platform-specific details.
    if windows {
        cfg.define("WIN32", None)
            .define("USE_THREADS_WIN32", None)
            .define("HAVE_IOCTLSOCKET_FIONBIO", None)
            .define("USE_WINSOCK", None)
            .file("curl/lib/bufref.c")
            .file("curl/lib/system_win32.c")
            .file("curl/lib/vauth/digest_sspi.c")
            .file("curl/lib/curlx/multibyte.c")
            .file("curl/lib/curlx/version_win32.c")
            .file("curl/lib/curlx/winapi.c");

        if cfg!(feature = "spnego") {
            cfg.file("curl/lib/vauth/spnego_sspi.c");
        }
    } else {
        cfg.define("RECV_TYPE_ARG1", "int")
            .define("HAVE_PTHREAD_H", None)
            .define("HAVE_ARPA_INET_H", None)
            .define("HAVE_ERRNO_H", None)
            .define("HAVE_FCNTL_H", None)
            .define("HAVE_NETDB_H", None)
            .define("HAVE_NETINET_IN_H", None)
            .define("HAVE_NETINET_TCP_H", None)
            .define("HAVE_POLL_H", None)
            .define("HAVE_FCNTL_O_NONBLOCK", None)
            .define("HAVE_SYS_SELECT_H", None)
            .define("HAVE_SYS_STAT_H", None)
            .define("HAVE_SYS_TIME_H", None)
            .define("HAVE_UNISTD_H", None)
            .define("HAVE_RECV", None)
            .define("HAVE_SELECT", None)
            .define("HAVE_SEND", None)
            .define("HAVE_SOCKET", None)
            .define("HAVE_STERRROR_R", None)
            .define("HAVE_SOCKETPAIR", None)
            .define("HAVE_STRUCT_TIMEVAL", None)
            .define("HAVE_SYS_UN_H", None)
            .define("USE_THREADS_POSIX", None)
            .define("USE_UNIX_SOCKETS", None)
            .define("RECV_TYPE_ARG2", "void*")
            .define("RECV_TYPE_ARG3", "size_t")
            .define("RECV_TYPE_ARG4", "int")
            .define("RECV_TYPE_RETV", "ssize_t")
            .define("SEND_QUAL_ARG2", "const")
            .define("SEND_TYPE_ARG1", "int")
            .define("SEND_TYPE_ARG2", "void*")
            .define("SEND_TYPE_ARG3", "size_t")
            .define("SEND_TYPE_ARG4", "int")
            .define("SEND_TYPE_RETV", "ssize_t")
            .define("SIZEOF_CURL_OFF_T", "8")
            .define("SIZEOF_INT", "4")
            .define("SIZEOF_SHORT", "2");

        if target.contains("-apple-") {
            cfg.define("__APPLE__", None)
                .define("HAVE_MACH_ABSOLUTE_TIME", None);
        } else {
            cfg.define("HAVE_CLOCK_GETTIME_MONOTONIC", None)
                .define("HAVE_GETTIMEOFDAY", None)
                // poll() on various versions of macOS are janky, so only use it
                // on non-macOS unix-likes. This matches the official default
                // build configuration as well.
                .define("HAVE_POLL_FINE", None);
        }

        if cfg!(feature = "spnego") {
            cfg.define("HAVE_GSSAPI", None)
                .file("curl/lib/curl_gssapi.c")
                .file("curl/lib/socks_gssapi.c")
                .file("curl/lib/vauth/spnego_gssapi.c");
            if let Some(path) = env::var_os("GSSAPI_ROOT") {
                let path = PathBuf::from(path);
                cfg.include(path.join("include"));
            }

            // Link against the MIT gssapi library. It might be desirable to add support for
            // choosing between MIT and Heimdal libraries in the future.
            println!("cargo:rustc-link-lib=gssapi_krb5");
        }

        let width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        cfg.define("SIZEOF_SSIZE_T", Some(&(width / 8).to_string()[..]));
        cfg.define("SIZEOF_SIZE_T", Some(&(width / 8).to_string()[..]));
        cfg.define("SIZEOF_LONG", Some(&(width / 8).to_string()[..]));

        cfg.flag("-fvisibility=hidden");
    }

    cfg.compile("curl");

    if windows {
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=crypt32");
    }

    // Illumos/Solaris requires explicit linking with libnsl
    if target.contains("solaris") {
        println!("cargo:rustc-link-lib=nsl");
    }

    if target.contains("-apple-") {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreServices");
        println!("cargo:rustc-link-lib=framework=SystemConfiguration");
    }
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> bool {
    false
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> bool {
    // the import library for the dll is called libcurl_imp
    let mut successful_probe_details = match vcpkg::Config::new()
        .lib_names("libcurl_imp", "libcurl")
        .emit_includes(true)
        .probe("curl")
    {
        Ok(details) => Some(details),
        Err(e) => {
            println!("first run of vcpkg did not find libcurl: {}", e);
            None
        }
    };

    if successful_probe_details.is_none() {
        match vcpkg::Config::new()
            .lib_name("libcurl")
            .emit_includes(true)
            .probe("curl")
        {
            Ok(details) => successful_probe_details = Some(details),
            Err(e) => println!("second run of vcpkg did not find libcurl: {}", e),
        }
    }

    if successful_probe_details.is_some() {
        // Found libcurl which depends on openssl, libssh2 and zlib
        // in the a default vcpkg installation. Probe for them
        // but do not fail if they are not present as we may be working
        // with a customized vcpkg installation.
        vcpkg::Config::new()
            .lib_name("libeay32")
            .lib_name("ssleay32")
            .probe("openssl")
            .ok();

        vcpkg::probe_package("libssh2").ok();

        vcpkg::Config::new()
            .lib_names("zlib", "zlib1")
            .probe("zlib")
            .ok();

        println!("cargo:rustc-link-lib=crypt32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=wldap32");
        return true;
    }
    false
}

fn try_pkg_config() -> bool {
    let mut cfg = pkg_config::Config::new();
    cfg.cargo_metadata(false);
    let lib = match cfg.probe("libcurl") {
        Ok(lib) => lib,
        Err(e) => {
            println!(
                "Couldn't find libcurl from pkgconfig ({:?}), \
                 compiling it from source...",
                e
            );
            return false;
        }
    };

    // Not all system builds of libcurl have http2 features enabled, so if we've
    // got a http2-requested build then we may fall back to a build from source.
    if cfg!(feature = "http2") && !curl_config_reports_http2() {
        return false;
    }

    // Re-find the library to print cargo's metadata, then print some extra
    // metadata as well.
    cfg.cargo_metadata(true).probe("libcurl").unwrap();
    for path in lib.include_paths.iter() {
        println!("cargo:include={}", path.display());
    }
    true
}

fn xcode_major_version() -> Option<u8> {
    let status = Command::new("xcode-select").arg("-p").status().ok()?;
    if status.success() {
        let output = Command::new("xcodebuild").arg("-version").output().ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("xcode version: {}", stdout);
            let mut words = stdout.split_whitespace();
            if words.next()? == "Xcode" {
                let version = words.next()?;
                return version[..version.find('.')?].parse().ok();
            }
        }
    }
    println!("unable to determine Xcode version, assuming >= 9");
    None
}

fn curl_config_reports_http2() -> bool {
    let output = Command::new("curl-config").arg("--features").output();
    let output = match output {
        Ok(out) => out,
        Err(e) => {
            println!("failed to run curl-config ({}), building from source", e);
            return false;
        }
    };
    if !output.status.success() {
        println!("curl-config failed: {}", output.status);
        return false;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("HTTP2") {
        println!(
            "failed to find http-2 feature enabled in pkg-config-found \
             libcurl, building from source"
        );
        return false;
    }

    true
}

fn macos_link_search_path() -> Option<String> {
    let output = cc::Build::new()
        .get_compiler()
        .to_command()
        .arg("--print-search-dirs")
        .output()
        .ok()?;
    if !output.status.success() {
        println!(
            "failed to run 'clang --print-search-dirs', continuing without a link search path"
        );
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("libraries: =") {
            let path = line.split('=').nth(1)?;
            if !path.is_empty() {
                return Some(format!("{}/lib/darwin", path));
            }
        }
    }

    println!("failed to determine link search path, continuing without it");
    None
}
