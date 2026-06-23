# © Realix > Makefile: Bootloader
# (21.06.26) v0.07
# ================

# Конфигурация
BUILD_DIR ?= $(abspath ../../build)
SRC_DIR ?= $(abspath ../../source)
ASM = nasm
ASMFLAGS = -f bin -i $(SRC_DIR)

.PHONY: all clean

# Запуск по умолчанию
all: bootix initrix


# Сборка загрузчика (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin:
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/bootix.asm -o $(BUILD_DIR)/bootix.bin


# Сборка инициализатора (bin)
initrix: $(BUILD_DIR)/initrix.bin

$(BUILD_DIR)/initrix.bin:
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/initrix.asm -o $(BUILD_DIR)/initrix.bin


# Очистка
clean:
	rm -f $(BUILD_DIR)/bootix.bin $(BUILD_DIR)/initrix.bin