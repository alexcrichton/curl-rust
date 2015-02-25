#![feature(io, env, path, core, process, fs, old_path)]

extern crate "pkg-config" as pkg_config;

use std::env;
use std::io::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();

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

    let mut cflags = env::var("CFLAGS").unwrap_or(String::new());
    let windows = target.contains("windows");
    cflags.push_str(" -ffunction-sections -fdata-sections");

    if target.contains("i686") {
        cflags.push_str(" -m32");
    } else if target.as_slice().contains("x86_64") {
        cflags.push_str(" -m64");
    }
    if !target.contains("i686") {
        cflags.push_str(" -fPIC");
    }

    let src = env::current_dir().unwrap();
    let dst = PathBuf::new(&env::var_os("OUT_DIR").unwrap());

    let _ = fs::create_dir(&dst.join("build"));

    let mut config_opts = Vec::new();
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
    fs::create_dir(&dst.join("lib/pkgconfig")).unwrap();

    // Which one does windows generate? Who knows!
    let p1 = dst.join("build/lib/.libs/libcurl.a");
    let p2 = dst.join("build/lib/.libs/libcurl.lib");
    if p1.exists() {
        fs::rename(&p1, &dst.join("lib/libcurl.a")).unwrap();
    } else {
        fs::rename(&p2, &dst.join("lib/libcurl.a")).unwrap();
    }
    fs::rename(&dst.join("build/libcurl.pc"),
               &dst.join("lib/pkgconfig/libcurl.pc")).unwrap();

    if windows {
        println!("cargo:rustc-flags=-l ws2_32");
    }
    println!("cargo:rustc-flags=-L {}/lib -l curl:static", dst.display());
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    assert!(cmd.status()
               .unwrap()
               .success());
}

fn make() -> &'static str {
    if cfg!(target_os = "freebsd") {"gmake"} else {"make"}
}

fn which(cmd: &str) -> Option<PathBuf> {
    let cmd = format!("{}{}", cmd, env::consts::EXE_SUFFIX);
    let paths = env::var_os("PATH").unwrap();
    env::split_paths(&paths).map(|p| PathBuf::new(p.as_str().unwrap()))
        .map(|p| p.join(&cmd)).find(|p| p.exists())
}
