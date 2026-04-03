#!/bin/sh

set -ex

# Note that `-Clink-dead-code` is passed here to suppress `--gc-sections` to
# help confirm that we're compiling everything necessary for curl itself.
export RUSTFLAGS=-Clink-dead-code

# On macOS, test with the deployment target set to Rust's minimum:
# https://doc.rust-lang.org/rustc/platform-support/apple-darwin.html#os-version
if [ "$TARGET" = "x86_64-apple-darwin" ]; then
  export MACOSX_DEPLOYMENT_TARGET=10.12
fi
if [ "$TARGET" = "aarch64-apple-darwin" ]; then
  export MACOSX_DEPLOYMENT_TARGET=11.0
fi

# For musl on CI always use openssl-src dependency and build from there.
if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
  features="--features static-ssl"
fi

cargo test --target $TARGET --no-run $features --release
# First test with no extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl $features --release
# Then with rustls TLS backend.
#
# Note: Cross-compiling rustls on windows doesn't work due to requiring some
# NASM build stuff in aws_lc_rs, which may soon be fixed by
# https://github.com/aws/aws-lc-rs/pull/528.
#
# Compiling on i686-windows requires nasm to be installed (other platforms
# have pre-compiled object files), which is just slightly too much
# inconvenience for me.
if [ "$TARGET" != "x86_64-pc-windows-gnu" ] && [ "$TARGET" != "i686-pc-windows-msvc" ]
then
    cargo test --target $TARGET --no-run --features rustls,static-curl $features --release
fi
# Then with all extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl,protocol-ftp,ntlm $features --release
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET $features --release
    cargo test --target $TARGET --features static-curl $features --release
    cargo test --target $TARGET --features static-curl,protocol-ftp $features --release
    cargo test --target $TARGET --features static-curl,http2 $features --release

    cargo run --manifest-path systest/Cargo.toml --target $TARGET $features --release
    cargo run --manifest-path systest/Cargo.toml --target $TARGET --features curl-sys/static-curl,curl-sys/protocol-ftp $features --release

    cargo doc --no-deps --target $TARGET $features --release
    cargo doc --no-deps -p curl-sys --target $TARGET $features --release
fi
