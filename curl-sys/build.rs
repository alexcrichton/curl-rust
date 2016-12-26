extern crate pkg_config;
extern crate gcc;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::io::ErrorKind;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(e) => panic!("{} return the error {}", stringify!($e), e),
    })
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let src = env::current_dir().unwrap();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let windows = target.contains("windows");

    // OSX ships libcurl by default, so we just use that version
    // unconditionally.
    if target.contains("apple") {
        return println!("cargo:rustc-flags=-l curl");
    }

    // Illumos/Solaris requires explicit linking with libnsl
    if target.contains("solaris") {
        println!("cargo:rustc-flags=-l nsl");
    }

    // Next, fall back and try to use pkg-config if its available.
    if !target.contains("windows") {
        match pkg_config::find_library("libcurl") {
            Ok(lib) => {
                for path in lib.include_paths.iter() {
                    println!("cargo:include={}", path.display());
                }
                return
            }
            Err(e) => println!("Couldn't find libcurl from \
                               pkgconfig ({:?}), compiling it from source...", e),
        }
    }

    if !Path::new("curl/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
                                   .status();
    }

    println!("cargo:rustc-link-search={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=curl");
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
    if windows {
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=crypt32");
    }

    // MSVC builds are just totally different
    if target.contains("msvc") {
        return build_msvc(&target);
    }

    let openssl_root = register_dep("OPENSSL");
    let zlib_root = register_dep("Z");

    let cfg = gcc::Config::new();
    let compiler = cfg.get_compiler();

    let _ = fs::create_dir(&dst.join("build"));

    let mut cmd = Command::new("sh");
    let mut cflags = OsString::new();
    for arg in compiler.args() {
        cflags.push(arg);
        cflags.push(" ");
    }

    // Can't run ./configure directly on msys2 b/c we're handing in
    // Windows-style paths (those starting with C:\), but it chokes on those.
    // For that reason we build up a shell script with paths converted to
    // posix versions hopefully...
    cmd.env("CC", compiler.path())
       .env("CFLAGS", cflags)
       .env("VERBOSE", "1")
       .env("LD", msys_compatible(&which("ld").unwrap()))
       .current_dir(&dst.join("build"))
       .arg(msys_compatible(&src.join("curl/configure")));

    // For now this build script doesn't support paths with spaces in them. This
    // is arguably a but in curl's configure script, but we could also try to
    // paper over it by using a tmp directory which *doesn't* have spaces in it.
    // As of now though that's not implemented so just give a nicer error for
    // the time being.
    let wants_space_error = windows &&
        (dst.to_str().map(|s| s.contains(" ")).unwrap_or(false) ||
         src.to_str().map(|s| s.contains(" ")).unwrap_or(false));
    if wants_space_error {
        panic!("\n\nunfortunately ./configure of libcurl is known to \
                fail if there's a space in the path to the current \
                directory\n\n\
                there's a space in either\n  {}\n  {}\nand this will cause the \
                build to fail\n\n\
                the MSVC build should work with a directory that has \
                spaces in it, and it would also work to move this to a \
                different directory without spaces\n\n",
               src.display(), dst.display())
    }

    if windows {
        cmd.arg("--with-winssl");
    } else {
        cmd.arg("--without-ca-bundle");
        cmd.arg("--without-ca-path");
    }
    if let Some(root) = openssl_root {
        cmd.arg(format!("--with-ssl={}", msys_compatible(&root)));
    }
    if let Some(root) = zlib_root {
        cmd.arg(format!("--with-zlib={}", msys_compatible(&root)));
    }
    cmd.arg("--enable-static=yes");
    cmd.arg("--enable-shared=no");
    match &env::var("PROFILE").unwrap()[..] {
        "bench" | "release" => {
            cmd.arg("--enable-optimize");
        }
        _ => {
            cmd.arg("--enable-debug");
            cmd.arg("--disable-optimize");
        }
    }
    cmd.arg(format!("--prefix={}", msys_compatible(&dst)));

    if target != host &&
       (!target.contains("windows") || !host.contains("windows")) {
        cmd.arg(format!("--host={}", host));
        cmd.arg(format!("--target={}", target));
    }

    cmd.arg("--without-librtmp");
    cmd.arg("--without-libidn");
    cmd.arg("--without-libssh2");
    cmd.arg("--without-nghttp2");
    cmd.arg("--disable-ldap");
    cmd.arg("--disable-ldaps");
    cmd.arg("--disable-ftp");
    cmd.arg("--disable-rtsp");
    cmd.arg("--disable-dict");
    cmd.arg("--disable-telnet");
    cmd.arg("--disable-tftp");
    cmd.arg("--disable-pop3");
    cmd.arg("--disable-imap");
    cmd.arg("--disable-smtp");
    cmd.arg("--disable-gopher");
    cmd.arg("--disable-manual");
    cmd.arg("--disable-smb");
    cmd.arg("--disable-sspi");

    run(&mut cmd, "sh");
    run(Command::new(make())
                .arg(&format!("-j{}", env::var("NUM_JOBS").unwrap()))
                .current_dir(&dst.join("build")), "make");
    run(Command::new(make())
                .arg("install")
                .current_dir(&dst.join("build")), "make");
}

fn run(cmd: &mut Command, program: &str) {
    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            fail(&format!("failed to execute command: {}\nis `{}` not installed?",
                          e, program));
        }
        Err(e) => fail(&format!("failed to execute command: {}", e)),
    };
    if !status.success() {
        fail(&format!("command did not execute successfully, got: {}", status));
    }
}

fn fail(s: &str) -> ! {
    panic!("\n{}\n\nbuild script failed, must exit now", s)
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

fn msys_compatible(path: &Path) -> String {
    let path = path.to_str().unwrap();
    if !cfg!(windows) {
        return path.to_string()
    }
    path.replace("C:\\", "/c/")
        .replace("\\", "/")
}

fn register_dep(dep: &str) -> Option<PathBuf> {
    if let Some(s) = env::var_os(&format!("DEP_{}_ROOT", dep)) {
        prepend("PKG_CONFIG_PATH", Path::new(&s).join("lib/pkgconfig"));
        return Some(s.into())
    }
    if let Some(s) = env::var_os(&format!("DEP_{}_INCLUDE", dep)) {
        let root = Path::new(&s).parent().unwrap();
        env::set_var(&format!("DEP_{}_ROOT", dep), root);
        let path = root.join("lib/pkgconfig");
        if path.exists() {
            prepend("PKG_CONFIG_PATH", path);
            return Some(root.to_path_buf())
        }
    }

    return None;

    fn prepend(var: &str, val: PathBuf) {
        let prefix = env::var(var).unwrap_or(String::new());
        let mut v = vec![val];
        v.extend(env::split_paths(&prefix));
        env::set_var(var, &env::join_paths(v).unwrap());
    }
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

    let features = env::var("CARGO_CFG_TARGET_FEATURE")
                      .unwrap_or(String::new());
    if features.contains("crt-static") {
        cmd.arg("RTLIBCFG=static");
    }

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
    run(&mut cmd, "nmake");

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
