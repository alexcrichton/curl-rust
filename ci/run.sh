#!/bin/sh

set -ex

# install cmake. cmake required for zlib-ng build, and old cmake may not work,
# so we install the latest one. this step only run for ci docker environment to
# avoid local development environment pollution.
if [ -f /.dockerenv ]; then
    CMAKE_URL=https://github.com/Kitware/CMake/releases/download/v3.25.0/cmake-3.25.0-linux-x86_64.tar.gz

    # check curl or wget is installed for downloading cmake
    if command -v curl >/dev/null 2>&1; then
        DL="curl -sSL"
    elif command -v wget >/dev/null 2>&1; then
        DL="wget -qO-"
    else
        echo "curl or wget is required to download cmake"
        exit 1
    fi

    if [ ! -f /opt/cmake/bin/cmake ]; then  # check cmake is installed (avoid multiple install)
        # download cmake and install
        mkdir -p /opt/cmake
        $DL $CMAKE_URL | tar -C /opt/cmake -xz --strip-components=1
        export PATH=/opt/cmake/bin:$PATH
    fi
fi

cargo test --target $TARGET --no-run
# First test with no extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl
# Then with rustls TLS backend.
cargo test --target $TARGET --no-run --features rustls,static-curl
# Then with all extra protocols enabled.
cargo test --target $TARGET --no-run --features static-curl,protocol-ftp,ntlm
if [ -z "$NO_RUN" ]; then
    cargo test --target $TARGET
    cargo test --target $TARGET --features static-curl
    cargo test --target $TARGET --features static-curl,protocol-ftp
    cargo test --target $TARGET --features static-curl,zlib
    cargo test --target $TARGET --features static-curl,zlib-ng-compat

    # Note that `-Clink-dead-code` is passed here to suppress `--gc-sections` to
    # help confirm that we're compiling everything necessary for curl itself.
    RUSTFLAGS=-Clink-dead-code \
    cargo run --manifest-path systest/Cargo.toml --target $TARGET
    RUSTFLAGS=-Clink-dead-code \
    cargo run --manifest-path systest/Cargo.toml --target $TARGET --features curl-sys/static-curl,curl-sys/protocol-ftp

    cargo doc --no-deps --target $TARGET
    cargo doc --no-deps -p curl-sys --target $TARGET
fi
