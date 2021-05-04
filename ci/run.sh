#!/bin/sh

set -ex

cargo test --target $TARGET --no-run
# First test with no extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl
# Then with all extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl,protocol-ftp
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET
    cargo test --target $TARGET --features static-curl
    cargo test --target $TARGET --features static-curl,protocol-ftp
    cargo run --manifest-path systest/Cargo.toml --target $TARGET
    cargo run --manifest-path systest/Cargo.toml --target $TARGET --features curl-sys/static-curl,curl-sys/protocol-ftp
    cargo doc --no-deps --target $TARGET
    cargo doc --no-deps -p curl-sys --target $TARGET
fi
