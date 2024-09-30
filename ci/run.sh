#!/bin/sh

set -ex

# For musl on CI always use openssl-src dependency and build from there.
if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
  features="--features static-ssl"
fi

cargo test --target $TARGET --no-run $features
# First test with no extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl $features
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
    cargo test --target $TARGET --no-run --features rustls,static-curl $features
fi
# Then with all extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl,protocol-ftp,ntlm $features
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET $features
    cargo test --target $TARGET --features static-curl $features
    cargo test --target $TARGET --features static-curl,protocol-ftp $features

    # Note that `-Clink-dead-code` is passed here to suppress `--gc-sections` to
    # help confirm that we're compiling everything necessary for curl itself.
    RUSTFLAGS=-Clink-dead-code \
    cargo run --manifest-path systest/Cargo.toml --target $TARGET $features
    RUSTFLAGS=-Clink-dead-code \
    cargo run --manifest-path systest/Cargo.toml --target $TARGET --features curl-sys/static-curl,curl-sys/protocol-ftp $features

    cargo doc --no-deps --target $TARGET $features
    cargo doc --no-deps -p curl-sys --target $TARGET $features
fi
