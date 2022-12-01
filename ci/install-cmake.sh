#!/bin/sh
#
# Install cmake. cmake required for zlib-ng build, and old cmake may not work,
# so we install the latest one.

set -ex

CMAKE_VERSION=3.25.0
CMAKE_URL=https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz

if command -v curl >/dev/null 2>&1; then
    DL="curl -sSL"
elif command -v wget >/dev/null 2>&1; then
    DL="wget -qO-"
else
    # trying to install wget (compatible with ubuntu and centos docker image)
    echo "curl or wget not found, trying to install wget..."
    if command -v apt-get >/dev/null 2>&1; then  # ubuntu
        apt-get update
        apt-get install -y --no-install-recommends wget
    elif command -v yum >/dev/null 2>&1; then  # centos
        yum install -y wget
    else
        echo "No package manager found, cannot install wget to download cmake"
        exit 1
    fi
    DL="wget -qO-"
fi

# download cmake and install
echo "installing cmake..."
mkdir -p /opt/cmake
$DL $CMAKE_URL | tar -C /opt/cmake -xz --strip-components=1
ln -s /opt/cmake/bin/cmake /usr/bin/cmake
