# OSX always has libcurl, so we do nothing on osx
ifneq ($(findstring darwin,$(TARGET)),)
all:
else

ifneq ($(findstring i686,$(TARGET)),)
export CFLAGS := $(CFLAGS) -m32 -ffunction-sections -fdata-sections
else ifneq ($(findstring x86_64,$(TARGET)),)
export CFLAGS := $(CFLAGS) -fPIC -m64 -ffunction-sections -fdata-sections
else
export CFLAGS := $(CFLAGS) -fPIC -ffunction-sections -fdata-sections
endif

ifneq ($(findstring mingw,$(TARGET)),)
# Otherwise apparently the makefiles get corrupted and aren't valid
export LD=$(shell which ld)
OPTS := --with-winssl
endif

OPTS := $(OPTS) \
        --enable-static=yes \
        --enable-shared=no \
	--disable-ldap --disable-ldaps --disable-ftp --disable-rtsp \
	--disable-dict --disable-telnet --disable-tftp --disable-pop3 \
	--disable-imap --disable-smtp --disable-gopher --disable-manual \
	--enable-optimize --without-librtmp --without-libidn \
        --prefix="$(OUT_DIR)"

PWD := $(CURDIR)
LIBCURL := curl-7.37.1

# If libcurl exists locally, don't do anything. Otherwise go and build curl
all:
	pkg-config --exists libcurl || $(MAKE) build_curl

build_curl:
	mkdir -p "$(OUT_DIR)/build"
	(cd "$(OUT_DIR)/build" && "$(PWD)/$(LIBCURL)/configure" $(OPTS))
	make -C "$(OUT_DIR)/build"
	cp "$(OUT_DIR)/build/lib/.libs/libcurl.a" "$(OUT_DIR)/libcurl.a" || exit 0
	cp "$(OUT_DIR)/build/lib/.libs/libcurl.lib" "$(OUT_DIR)/libcurl.a" || exit 0
	cp "$(OUT_DIR)/build/libcurl.pc" "$(OUT_DIR)/libcurl.pc"

endif
