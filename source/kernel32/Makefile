# © Realix > Makefile: Kernel32
# (21.06.26) v0.07
# ================

# Конфигурация
TARGET_BIN = ../../build/kernel32.bin
CARGO_OUTPUT = ../../target/i386/release/kernel32

.PHONY: all clean

# Запуск по умолчанию
all: kernel32

# Сборка 32-битного ядра (Rust)
kernel32: $(TARGET_BIN)

$(TARGET_BIN):
	cargo build --release --manifest-path ../../Cargo.toml
	rust-objcopy -O binary $(CARGO_OUTPUT) $(TARGET_BIN)

# Очистка
clean:
	cargo clean --manifest-path ../../Cargo.toml