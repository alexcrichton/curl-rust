# Root of the project
ROOT = $(dir $(firstword $(MAKEFILE_LIST)))

# Path to rustc executable
RUSTC ?= rustc

# Flags to pass rustc
RUSTC_FLAGS ?=

SRC = $(shell find $(ROOT)src -name '*.rs')

TARGET ?= target

LIBCURLRUST = $(TARGET)/libcurl.timestamp

LIBCURLRUST_TEST = $(TARGET)/libcurl-test

all: $(LIBCURLRUST)

test: $(LIBCURLRUST_TEST)
	$(LIBCURLRUST_TEST) $(only)

$(LIBCURLRUST): $(SRC) | $(TARGET)
	$(RUSTC) --out-dir $(TARGET) --crate-type=rlib src/lib.rs
	touch $@

$(LIBCURLRUST_TEST): $(SRC) | $(TARGET)
	$(RUSTC) --test -o $@ src/lib.rs

hax: $(SRC)
	$(RUSTC) --cfg hax -o $@ --crate-type=bin src/lib.rs

clean:
	rm -f $(TARGET)/libcurl*

$(TARGET):
	mkdir -p $@

.PHONY: all clean distclean test

.SUFFIXES:
