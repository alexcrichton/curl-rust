#!/bin/sh

set -ex

cargo test --target $TARGET --no-run
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET
    cargo run --manifest-path systest/Cargo.toml --target $TARGET
    cargo doc --no-deps --target $TARGET
    cargo doc --no-deps -p curl-sys --target $TARGET
fi
