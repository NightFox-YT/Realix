# © Realix > Makefile: Kernel32
# Исправленная версия
# ===================

ROOT_DIR    := $(abspath ../..)
BUILD_DIR   ?= $(ROOT_DIR)/build

TARGET_BIN   := $(BUILD_DIR)/kernel32.bin
CARGO_OUTPUT := $(ROOT_DIR)/target/i386/release/kernel32
OBJCOPY      := rust-objcopy

.PHONY: all kernel32 clean

all: kernel32

kernel32: $(TARGET_BIN)

$(BUILD_DIR):
	mkdir -p $@

$(TARGET_BIN): | $(BUILD_DIR)
	cd $(ROOT_DIR) && \
	    cargo +nightly build \
	        --release \
	        --manifest-path Cargo.toml \
	        -Z json-target-spec

	$(OBJCOPY) -O binary \
	    $(CARGO_OUTPUT) \
	    $(TARGET_BIN)

clean:
	cd $(ROOT_DIR) && \
	    cargo +nightly clean \
	        --manifest-path Cargo.toml

	rm -f $(TARGET_BIN)
