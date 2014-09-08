ifneq ($(findstring i686,$(TARGET)),)
export CFLAGS := $(CFLAGS) -m32 -ffunction-sections -fdata-sections
else
export CFLAGS := $(CFLAGS) -fPIC -m64 -ffunction-sections -fdata-sections
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
	--enable-optimize \
        --prefix="$(OUT_DIR)"

PWD := $(CURDIR)
LIBCURL := curl-7.37.1

all:
	rm -rf "$(OUT_DIR)/include" "$(OUT_DIR)/lib" "$(OUT_DIR)/build"
	mkdir -p "$(OUT_DIR)/build" "$(OUT_DIR)/lib/pkgconfig"
	[ -f "$(OUT_DIR)/build/Makefile" ] && \
		make -C "$(OUT_DIR)/build" distclean || true
	(cd "$(OUT_DIR)/build" && "$(PWD)/$(LIBCURL)/configure" $(OPTS))
	make -C "$(OUT_DIR)/build/src"
	make -C "$(OUT_DIR)/build/src" install
