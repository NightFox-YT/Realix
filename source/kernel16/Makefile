# © Realix > Makefile: Kernel16
# (11.07.26) v0.09
# ================

# Конфигурация
BUILD_DIR  ?= $(abspath ../../build)
SRC_DIR    ?= $(abspath ../../source)
TARGET_BIN := $(BUILD_DIR)/kernel16.bin
ASM        := nasm
ASMFLAGS   := -f bin -i $(SRC_DIR)

# Источники, подключаемые через %include в main.asm
KERNEL16_SRC := $(shell find $(SRC_DIR)/kernel16 -name '*.asm') \
                $(SRC_DIR)/shared/config.asm \
                $(SRC_DIR)/network/rtl8139.asm \
                $(SRC_DIR)/bios-api/memory/high.asm \
                $(SRC_DIR)/bios-api/memory/low.asm

.PHONY: all kernel16 clean

# Запуск по умолчанию
all: kernel16

# Сборка ядра (bin)
kernel16: $(TARGET_BIN)

$(TARGET_BIN): $(KERNEL16_SRC) | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/kernel16/main.asm -o $(TARGET_BIN)

$(BUILD_DIR):
	mkdir -p $@

# Очистка
clean:
	rm -f $(TARGET_BIN)
