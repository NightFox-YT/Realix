# © Realix > Makefile: Kernel16
# Исправленная версия
# ===================

BUILD_DIR ?= $(abspath ../../build)
SRC_DIR   ?= $(abspath ../../source)

ASM      := nasm
ASMFLAGS := -f bin -i $(SRC_DIR)/

TARGET := $(BUILD_DIR)/kernel16.bin

.PHONY: all kernel16 clean

all: kernel16

kernel16: $(TARGET)

$(BUILD_DIR):
	mkdir -p $@

$(TARGET): | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) \
	    $(SRC_DIR)/kernel16/main.asm \
	    -o $(TARGET)

clean:
	rm -f $(TARGET)
