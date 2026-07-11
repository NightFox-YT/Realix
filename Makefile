# © Realix > Makefile: Main
# Исправленная версия
# ===================

ASM       := nasm
SRC_DIR   := source
BUILD_DIR := build
ASMFLAGS  := -f bin -i $(SRC_DIR)/

.PHONY: all floppy bootix initrix kernel16 kernel32 run clean

all: floppy


# ============================================================
# ПОДГОТОВКА КАТАЛОГА СБОРКИ
# ============================================================

$(BUILD_DIR):
	mkdir -p $@


# ============================================================
# ИТОГОВЫЙ ОБРАЗ
# ============================================================

floppy: bootix initrix kernel16 kernel32
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mformat -i $(BUILD_DIR)/realix.img -f 1440 ::
	dd if=$(BUILD_DIR)/bootix.bin \
	   of=$(BUILD_DIR)/realix.img \
	   bs=512 count=1 conv=notrunc
	mcopy -o -i $(BUILD_DIR)/realix.img \
	   $(BUILD_DIR)/initrix.bin "::INITRIX.BIN"
	mcopy -o -i $(BUILD_DIR)/realix.img \
	   $(BUILD_DIR)/kernel16.bin "::KERNEL16.BIN"
	mcopy -o -i $(BUILD_DIR)/realix.img \
	   $(BUILD_DIR)/kernel32.bin "::KERNEL32.BIN"


# ============================================================
# BOOTIX
# ============================================================

bootix: | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) \
	    $(SRC_DIR)/bootloader/bootix.asm \
	    -o $(BUILD_DIR)/bootix.bin


# ============================================================
# INITRIX
# ============================================================

initrix: | $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) \
	    $(SRC_DIR)/bootloader/initrix.asm \
	    -o $(BUILD_DIR)/initrix.bin


# ============================================================
# KERNEL16
# ============================================================

kernel16: | $(BUILD_DIR)
	$(MAKE) -C $(SRC_DIR)/kernel16 \
	    BUILD_DIR=$(abspath $(BUILD_DIR)) \
	    SRC_DIR=$(abspath $(SRC_DIR))


# ============================================================
# KERNEL32
# ============================================================

kernel32: | $(BUILD_DIR)
	$(MAKE) -C $(SRC_DIR)/kernel32 \
	    BUILD_DIR=$(abspath $(BUILD_DIR))


# ============================================================
# ЗАПУСК
# ============================================================

run: floppy
	qemu-system-x86_64 \
	    -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy


# ============================================================
# ОЧИСТКА
# ============================================================

clean:
	$(MAKE) -C $(SRC_DIR)/kernel16 \
	    BUILD_DIR=$(abspath $(BUILD_DIR)) \
	    SRC_DIR=$(abspath $(SRC_DIR)) \
	    clean

	$(MAKE) -C $(SRC_DIR)/kernel32 \
	    BUILD_DIR=$(abspath $(BUILD_DIR)) \
	    clean

	rm -rf $(BUILD_DIR)
