#!/bin/sh
#
# Install cmake. cmake required for zlib-ng build, and old cmake may not work,
# so we install the latest one.

set -ex

CMAKE_VERSION=3.25.0
CMAKE_URL=https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz

mkdir -p /opt/cmake
curl -sSL $CMAKE_URL | tar -C /opt/cmake -xz --strip-components=1
ln -s /opt/cmake/bin/cmake /usr/bin/cmake
