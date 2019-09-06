#!/bin/sh

set -ex

cargo test --target $TARGET --no-run
cargo test --target $TARGET --no-run --features curl-sys/static-curl
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET
    cargo test --target $TARGET --features curl-sys/static-curl
    cargo run --manifest-path systest/Cargo.toml --target $TARGET
    cargo run --manifest-path systest/Cargo.toml --target $TARGET --features curl-sys/static-curl
    cargo doc --no-deps --target $TARGET
    cargo doc --no-deps -p curl-sys --target $TARGET
fi
