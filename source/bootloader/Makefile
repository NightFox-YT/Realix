# © Realix > Makefile: Bootloader
# (27.07.26) v0.1
# ================

# Конфигурация
BUILD_DIR ?= $(abspath ../../build)
SRC_DIR   ?= $(abspath ../../source)
ASM       := nasm
ASMFLAGS  := -f bin -i $(SRC_DIR)
ASM_SRC := $(shell find $(SRC_DIR) -name '*.asm')

.PHONY: all bootloader bootix initrix clean

# Запуск по умолчанию
all: bootix initrix

# Сборка Bootix
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: $(ASM_SRC) | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/bootix.asm -o $@

# Сборка Initrix
initrix: $(BUILD_DIR)/initrix.bin

$(BUILD_DIR)/initrix.bin: $(ASM_SRC) | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/initrix.asm -o $@

$(BUILD_DIR):
	mkdir -p $@

# Очистка
clean:
	rm -f $(BOOTIX_BIN) $(INITRIX_BIN)
