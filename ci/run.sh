#!/bin/sh

set -ex

cargo test --target $TARGET --no-run
# First test with no extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl
# Then with all extra protocols and features enabled.
cargo test --target $TARGET --no-run --features static-curl,protocol-ftp,upkeep_7_62_0,poll_7_68_0
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET
    cargo test --target $TARGET --features static-curl
    cargo test --target $TARGET --features static-curl,protocol-ftp,upkeep_7_62_0,poll_7_68_0

    # Note that `-Clink-dead-code` is passed here to suppress `--gc-sections` to
    # help confirm that we're compiling everything necessary for curl itself.
    RUSTFLAGS=-Clink-dead-code \
    cargo run --manifest-path systest/Cargo.toml --target $TARGET
    RUSTFLAGS=-Clink-dead-code \
    cargo run --manifest-path systest/Cargo.toml --target $TARGET --features \
        curl-sys/static-curl,curl-sys/protocol-ftp,curl-sys/upkeep_7_62_0,curl-sys/poll_7_68_0

    cargo doc --no-deps --target $TARGET
    cargo doc --no-deps -p curl-sys --target $TARGET
fi
