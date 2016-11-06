FROM ubuntu:16.04

RUN apt-get update
RUN apt-get install -y --no-install-recommends \
  gcc ca-certificates make libc6-dev curl \
  musl-tools

RUN \
  curl https://www.openssl.org/source/old/1.0.2/openssl-1.0.2g.tar.gz | tar xzf - && \
  cd openssl-1.0.2g && \
  CC=musl-gcc ./Configure --prefix=/openssl no-dso linux-x86_64 -fPIC && \
  make -j10 && \
  make install && \
  cd .. && \
  rm -rf openssl-1.0.2g

ENV OPENSSL_STATIC=1 \
    OPENSSL_DIR=/openssl
