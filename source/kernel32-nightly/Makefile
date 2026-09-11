# © Realix > Makefile: Kernel32 Nightly
# (07.09.26) v0.12
# ================
# ❗️ Собирается из общего корневого Cargo.toml (--bin kernel32-nightly), чтобы
#    путь к linker.ld в .cargo/config.toml резолвился так же, как для kernel32
#    (Оба указывают на один и тот же source/kernel32/linker.ld - разметка общая)

# Конфигурация
ROOT_DIR     := $(abspath ../..)
BUILD_DIR    ?= $(ROOT_DIR)/build
TARGET_BIN   := $(BUILD_DIR)/nightly.bin
CARGO_OUTPUT := $(ROOT_DIR)/target/i386/release/kernel32-nightly
OBJCOPY      := rust-objcopy

.PHONY: all kernel32-nightly clean

# Запуск по умолчанию
all: kernel32-nightly

# Сборка 32-битного ядра Nightly (Rust)
kernel32-nightly: | $(BUILD_DIR)
	cargo +nightly build --release --bin kernel32-nightly \
	--manifest-path $(ROOT_DIR)/Cargo.toml -Z json-target-spec
	$(OBJCOPY) -O binary $(CARGO_OUTPUT) $(TARGET_BIN)

$(BUILD_DIR):
	mkdir -p $@

# Очистка
clean:
	rm -f $(TARGET_BIN)
