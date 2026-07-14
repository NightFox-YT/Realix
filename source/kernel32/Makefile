# © Realix > Makefile: Kernel32
# (11.07.26) v0.09
# ================

# Конфигурация
ROOT_DIR     := $(abspath ../..)
BUILD_DIR    ?= $(ROOT_DIR)/build
TARGET_BIN   := $(BUILD_DIR)/kernel32.bin
CARGO_OUTPUT := $(ROOT_DIR)/target/i386/release/kernel32
OBJCOPY      := rust-objcopy

.PHONY: all kernel32 clean

# Запуск по умолчанию
all: kernel32

# Сборка 32-битного ядра (Rust)
kernel32: | $(BUILD_DIR)
	cargo +nightly build --release \
	--manifest-path $(ROOT_DIR)/Cargo.toml -Z json-target-spec
	$(OBJCOPY) -O binary $(CARGO_OUTPUT) $(TARGET_BIN)

$(BUILD_DIR):
	mkdir -p $@

# Очистка
clean:
	cargo +nightly clean --manifest-path $(ROOT_DIR)/Cargo.toml
	rm -f $(TARGET_BIN)
