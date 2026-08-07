# © Realix > Makefile: Kernel16
# (27.07.26) v0.1
# ================

# Конфигурация
BUILD_DIR  ?= $(abspath ../../build)
SRC_DIR    ?= $(abspath ../../source)
TARGET_BIN := $(BUILD_DIR)/kernel16.bin
ASM        := nasm
ASMFLAGS   := -f bin -i $(SRC_DIR)
ASM_SRC    := $(shell find $(SRC_DIR) -name '*.asm')

.PHONY: all kernel16 clean

# Запуск по умолчанию
all: kernel16

# Сборка ядра (bin)
kernel16: $(TARGET_BIN)

$(TARGET_BIN): $(ASM_SRC) | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/kernel16/main.asm -o $@

# Создание папки сборки
$(BUILD_DIR):
	mkdir -p $@

# Очистка
clean:
	rm -f $(TARGET_BIN)
