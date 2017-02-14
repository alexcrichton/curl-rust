FROM ubuntu:16.04

RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      gcc-multilib \
      ca-certificates \
      make \
      libc6-dev \
      libssl-dev:i386 \
      pkg-config

ENV PKG_CONFIG=i686-linux-gnu-pkg-config \
    PKG_CONFIG_ALLOW_CROSS=1
