# Realix build file
# Works on Linux, WSL and MSYS2.

ASM ?= nasm
QEMU ?= qemu-system-i386
SRC_DIR := source
BUILD_DIR := build
ASMFLAGS := -f bin -i$(SRC_DIR)/
ASM_SOURCES := $(shell find $(SRC_DIR) -name '*.asm')

BOOTIX_BIN := $(BUILD_DIR)/bootix.bin
INITRIX_BIN := $(BUILD_DIR)/initrix.bin
KERNEL_BIN := $(BUILD_DIR)/kernel.bin
IMAGE := $(BUILD_DIR)/realix.img

.PHONY: all floppy image bootix initrix kernel run run-debug check clean dirs

all: floppy

floppy image: $(IMAGE)

check:
	@command -v $(ASM) >/dev/null || { echo "Missing assembler: $(ASM)"; exit 1; }
	@command -v dd >/dev/null || { echo "Missing dd. Install coreutils."; exit 1; }
	@command -v mformat >/dev/null || { echo "Missing mformat. Install mtools."; exit 1; }
	@command -v mcopy >/dev/null || { echo "Missing mcopy. Install mtools."; exit 1; }
	@echo "Build tools found."

$(IMAGE): check $(BOOTIX_BIN) $(INITRIX_BIN) $(KERNEL_BIN)
	dd if=/dev/zero of=$(IMAGE) bs=512 count=2880 status=none
	mformat -i $(IMAGE) -f 1440 ::
	dd if=$(BOOTIX_BIN) of=$(IMAGE) conv=notrunc status=none
	mcopy -i $(IMAGE) $(INITRIX_BIN) "::INITRIX.BIN"
	mcopy -i $(IMAGE) $(KERNEL_BIN) "::KERNEL.BIN"
	@echo "Created $(IMAGE)"

bootix: $(BOOTIX_BIN)

$(BOOTIX_BIN): dirs $(ASM_SOURCES)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/bootix.asm -o $(BOOTIX_BIN)

initrix: $(INITRIX_BIN)

$(INITRIX_BIN): dirs $(ASM_SOURCES)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/initrix.asm -o $(INITRIX_BIN)

kernel: $(KERNEL_BIN)

$(KERNEL_BIN): dirs $(ASM_SOURCES)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/kernel16/kernel.asm -o $(KERNEL_BIN)

run: $(IMAGE)
	$(QEMU) -drive file=$(IMAGE),format=raw,if=floppy

run-debug: $(IMAGE)
	$(QEMU) -no-reboot -no-shutdown -d int,cpu_reset,guest_errors -D $(BUILD_DIR)/qemu-debug.log -drive file=$(IMAGE),format=raw,if=floppy

dirs:
	mkdir -p $(BUILD_DIR)

clean:
	rm -rf $(BUILD_DIR)
