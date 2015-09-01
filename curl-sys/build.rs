extern crate pkg_config;
extern crate gcc;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(e) => panic!("{} return the error {}", stringify!($e), e),
    })
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let src = env::current_dir().unwrap();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let windows = target.contains("windows");

    // OSX ships libcurl by default, so we just use that version
    // unconditionally.
    if target.contains("apple") {
        return println!("cargo:rustc-flags=-l curl");
    }

    // Next, fall back and try to use pkg-config if its available.
    match pkg_config::find_library("libcurl") {
        Ok(..) => return,
        Err(..) => {}
    }

    println!("cargo:rustc-link-search={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=curl");
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
    if windows {
        println!("cargo:rustc-flags=-l ws2_32");
    }

    // MSVC builds are just totally different
    if target.contains("msvc") {
        return build_msvc(&target);
    }

    let mut cflags = env::var("CFLAGS").unwrap_or(String::new());
    cflags.push_str(" -ffunction-sections -fdata-sections");

    if target.contains("i686") {
        cflags.push_str(" -m32");
    } else if target.contains("x86_64") {
        cflags.push_str(" -m64");
    }
    if !target.contains("i686") {
        cflags.push_str(" -fPIC");
    }

    let _ = fs::create_dir(&dst.join("build"));

    let mut config_opts = Vec::new();
    config_opts.push(format!("--host {}", target.replace("pc-windows-gnu", "w64-mingw32")));
    if windows {
        config_opts.push("--with-winssl".to_string());
    } else {
        config_opts.push("--without-ca-bundle".to_string());
        config_opts.push("--without-ca-path".to_string());

        match env::var("DEP_OPENSSL_ROOT") {
            Ok(s) => config_opts.push(format!("--with-ssl={}", s)),
            Err(..) => {}
        }
    }
    config_opts.push("--enable-static=yes".to_string());
    config_opts.push("--enable-shared=no".to_string());
    config_opts.push("--enable-optimize".to_string());
    config_opts.push(format!("--prefix={}", dst.display()));

    config_opts.push("--without-librtmp".to_string());
    config_opts.push("--without-libidn".to_string());
    config_opts.push("--without-libssh2".to_string());
    config_opts.push("--without-nghttp2".to_string());
    config_opts.push("--disable-ldap".to_string());
    config_opts.push("--disable-ldaps".to_string());
    config_opts.push("--disable-ftp".to_string());
    config_opts.push("--disable-rtsp".to_string());
    config_opts.push("--disable-dict".to_string());
    config_opts.push("--disable-telnet".to_string());
    config_opts.push("--disable-tftp".to_string());
    config_opts.push("--disable-pop3".to_string());
    config_opts.push("--disable-imap".to_string());
    config_opts.push("--disable-smtp".to_string());
    config_opts.push("--disable-gopher".to_string());
    config_opts.push("--disable-manual".to_string());

    // Can't run ./configure directly on msys2 b/c we're handing in
    // Windows-style paths (those starting with C:\), but it chokes on those.
    // For that reason we build up a shell script with paths converted to
    // posix versions hopefully...
    //
    // Also apparently the buildbots choke unless we manually set LD, who knows
    // why?!
    run(Command::new("sh")
                .env("CFLAGS", &cflags)
                .env("LD", &which("ld").unwrap())
                .current_dir(&dst.join("build"))
                .arg("-c")
                .arg(&format!("{} {}", src.join("curl/configure").display(),
                              config_opts.connect(" "))
                             .replace("C:\\", "/c/")
                             .replace("\\", "/")));
    run(Command::new(make())
                .arg(&format!("-j{}", env::var("NUM_JOBS").unwrap()))
                .current_dir(&dst.join("build")));

    // Don't run `make install` because apparently it's a little buggy on mingw
    // for windows.
    let _ = fs::create_dir_all(&dst.join("lib/pkgconfig"));

    // Which one does windows generate? Who knows!
    let p1 = dst.join("build/lib/.libs/libcurl.a");
    let p2 = dst.join("build/lib/.libs/libcurl.lib");
    if fs::metadata(&p1).is_ok() {
        t!(fs::copy(&p1, &dst.join("lib/libcurl.a")));
    } else {
        t!(fs::copy(&p2, &dst.join("lib/libcurl.a")));
    }
    t!(fs::copy(&dst.join("build/libcurl.pc"),
                  &dst.join("lib/pkgconfig/libcurl.pc")));
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    assert!(t!(cmd.status()).success());
}

fn make() -> &'static str {
    if cfg!(target_os = "freebsd") {"gmake"} else {"make"}
}

fn which(cmd: &str) -> Option<PathBuf> {
    let cmd = format!("{}{}", cmd, env::consts::EXE_SUFFIX);
    let paths = env::var_os("PATH").unwrap();
    env::split_paths(&paths).map(|p| p.join(&cmd)).find(|p| {
        fs::metadata(p).is_ok()
    })
}

fn build_msvc(target: &str) {
    let cmd = gcc::windows_registry::find(target, "nmake.exe");
    let mut cmd = cmd.unwrap_or(Command::new("nmake.exe"));
    let src = env::current_dir().unwrap();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let machine = if target.starts_with("x86_64") {
        "x64"
    } else if target.starts_with("i686") {
        "x86"
    } else {
        panic!("unknown msvc target: {}", target);
    };

    t!(fs::create_dir_all(dst.join("include/curl")));
    t!(fs::create_dir_all(dst.join("lib")));

    cmd.current_dir(src.join("curl/winbuild"));
    cmd.arg("/f").arg("Makefile.vc")
       .arg("MODE=static")
       .arg("ENABLE_IDN=yes")
       .arg("DEBUG=no")
       .arg("GEN_PDB=no")
       .arg("ENABLE_WINSSL=yes")
       .arg("ENABLE_SSPI=yes")
       .arg(format!("MACHINE={}", machine));

    if let Some(inc) = env::var_os("DEP_Z_ROOT") {
        let inc = PathBuf::from(inc);
        let mut s = OsString::from("WITH_DEVEL=");
        s.push(&inc);
        cmd.arg("WITH_ZLIB=static").arg(s);

        // the build system for curl expects this library to be called
        // zlib_a.lib, so make sure it's named correctly (where libz-sys just
        // produces zlib.lib)
        let _ = fs::remove_file(&inc.join("lib/zlib_a.lib"));
        t!(fs::hard_link(inc.join("lib/zlib.lib"), inc.join("lib/zlib_a.lib")));
    }
    run(&mut cmd);

    let name = format!("libcurl-vc-{}-release-static-zlib-static-\
                        ipv6-sspi-winssl", machine);
    let libs = src.join("curl/builds").join(name);

    t!(fs::copy(libs.join("lib/libcurl_a.lib"), dst.join("lib/curl.lib")));
    for f in t!(fs::read_dir(libs.join("include/curl"))) {
        let path = t!(f).path();
        let dst = dst.join("include/curl").join(path.file_name().unwrap());
        t!(fs::copy(path, dst));
    }
    t!(fs::remove_dir_all(src.join("curl/builds")));
    println!("cargo:rustc-link-lib=wldap32");
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=normaliz");
}
