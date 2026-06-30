# © Realix > Makefile: Kernel16
# (21.06.26) v0.07
# ================

# Конфигурация
BUILD_DIR ?= $(abspath ../../build)
SRC_DIR ?= $(abspath ../../source)
ASM = nasm
ASMFLAGS = -f bin -i $(SRC_DIR)

.PHONY: kernel16

# Запуск по умолчанию
all: kernel16

# Сборка ядра (bin)
kernel16: $(BUILD_DIR)/kernel16.bin

$(BUILD_DIR)/kernel16.bin:
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/kernel16/main.asm -o $(BUILD_DIR)/kernel16.bin


# Очистка
clean:
	rm -f $(BUILD_DIR)/kerne16.bin