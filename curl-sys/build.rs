extern crate "pkg-config" as pkg_config;

use std::os;
use std::io::{mod, fs, Command};
use std::io::process::InheritFd;

fn main() {
    let target = os::getenv("TARGET").unwrap();

    // OSX ships libcurl by default, so we just use that version
    // unconditionally.
    if target.contains("apple") {
        return println!("cargo:rustc-flags=-l curl");
    }

    // Next, fall back and try to use pkg-config if its available.
    match pkg_config::find_library("libcurl") {
        Ok(()) => return,
        Err(..) => {}
    }

    match os::getenv("DEP_OPENSSL_ROOT") {
        Some(s) => {
            let prefix = os::getenv("CMAKE_PREFIX_PATH").unwrap_or(String::new());
            let mut v = os::split_paths(prefix.as_slice());
            v.push(Path::new(s));
            os::setenv("CMAKE_PREFIX_PATH", os::join_paths(v.as_slice()).unwrap());
        }
        None => {}
    }

    let mut cflags = os::getenv("CFLAGS").unwrap_or(String::new());
    let windows = target.contains("windows");
    let mingw = windows && target.contains("gnu");
    cflags.push_str(" -ffunction-sections -fdata-sections");

    if target.contains("i686") {
        cflags.push_str(" -m32");
    } else if target.as_slice().contains("x86_64") {
        cflags.push_str(" -m64");
    }
    if !target.contains("i686") {
        cflags.push_str(" -fPIC");
    }

    let src = os::getcwd();
    let dst = Path::new(os::getenv("OUT_DIR").unwrap());

    let _ = fs::mkdir(&dst.join("build"), io::USER_DIR);

    let mut cmd = Command::new("cmake");
    if mingw {
        cmd.arg("-G").arg("Unix Makefiles");
    }
    run(cmd.arg(src.join("curl"))
           .args(&[
               "-DBUILD_CURL_EXE=OFF",
               "-DBUILD_CURL_TESTS=OFF",
               "-DCURL_STATICLIB=ON",
               "-DCURL_DISABLE_FTP=ON",
               "-DCURL_DISABLE_LDAP=ON",
               "-DCURL_DISABLE_TELNET=ON",
               "-DCURL_DISABLE_DICT=ON",
               "-DCURL_DISABLE_TFTP=ON",
               "-DCURL_DISABLE_RTSP=ON",
               "-DCURL_DISABLE_LDAPS=ON",
               "-DCURL_DISABLE_POP3=ON",
               "-DCURL_DISABLE_POP3=ON",
               "-DCURL_DISABLE_IMAP=ON",
               "-DCURL_DISABLE_SMTP=ON",
               "-DCURL_DISABLE_GOPHER=ON",
           ])
           .arg(format!("-DCMAKE_C_FLAGS={}", cflags))
           .arg("-DCMAKE_BUILD_TYPE=RelWithDebInfo")
           .arg(format!("-DCMAKE_INSTALL_PREFIX={}", dst.display()))
           .cwd(&dst.join("build")));
    run(Command::new("cmake")
                .arg("--build").arg(".")
                .arg("--target").arg("install")
                .cwd(&dst.join("build")));

    if windows {
        println!("cargo:rustc-flags=-l ws2_32");
    }
    println!("cargo:rustc-flags=-L {}/lib -l curl:static", dst.display());
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
}

fn run(cmd: &mut Command) {
    println!("running: {}", cmd);
    assert!(cmd.stdout(InheritFd(1))
               .stderr(InheritFd(2))
               .status()
               .unwrap()
               .success());

}
