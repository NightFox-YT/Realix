# © Realix > Makefile
# (18.06.26) v0.05
# ================

# Конфигурация инструментов
ASM = nasm
CC = gcc
LD = ld
DISK_IMG = realix_disk_fat32.img
INIT_BIN = $(BUILD_DIR)/initrix.bin
DISK_SIZE_MB = 64
INCLUDES = -I./source/libc -I./source/include -I./source/drivers-32/serial -I./source/kernel32/sysenter -I./source/kernel32/memory

CFLAGS = -m32 -g -ffreestanding -O2 -Wall -Wextra -fno-exceptions -D__realix__  -fno-builtin -fno-pic -fno-pie -nostdlib $(INCLUDES)

SRC_DIR = source
BUILD_DIR = build

C_SOURCES = $(wildcard $(SRC_DIR)/kernel32/*.c) \
            $(wildcard $(SRC_DIR)/kernel32/memory/*.c) \
            $(wildcard $(SRC_DIR)/kernel32/sysenter/*.c) \
			$(wildcard $(SRC_DIR)/drivers-32/keyboard/*.c) \
            $(wildcard $(SRC_DIR)/drivers-32/serial/*.c) \
			$(wildcard $(SRC_DIR)/vfs/*.c) \
			$(wildcard $(SRC_DIR)/drivers-32/ata/*.c) \
			$(wildcard $(SRC_DIR)/drivers-32/fat32/*.c) \
            $(wildcard $(SRC_DIR)/drivers-32/pci/*.c) \
			$(wildcard $(SRC_DIR)/drivers-32/rxbdph/*.c) \
			$(wildcard $(SRC_DIR)/libc/stdlib/*.c) \
            $(wildcard $(SRC_DIR)/interrupts/*.c) \
            $(wildcard $(SRC_DIR)/libc/atomic/*.c) \
            $(wildcard $(SRC_DIR)/libc/io/*.c) \
            $(wildcard $(SRC_DIR)/libc/string/*.c)

C_OBJECTS = $(patsubst $(SRC_DIR)/%.c, $(BUILD_DIR)/%.o, $(C_SOURCES))

.PHONY: all floppy bootix initrix kernel run clean always

all: floppy

floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix initrix kernel
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mformat -i $(BUILD_DIR)/realix.img -f 1440 ::
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/initrix.bin "::initrix.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/kernel.bin "::kernel.bin"

bootix: $(BUILD_DIR)/bootix.bin
$(BUILD_DIR)/bootix.bin: always
	$(ASM) -f bin -i $(SRC_DIR)/ $(SRC_DIR)/bootloader/bootix.asm -o $(BUILD_DIR)/bootix.bin

initrix: $(BUILD_DIR)/initrix.bin
$(BUILD_DIR)/initrix.bin: always
	$(ASM) -f bin -i $(SRC_DIR)/ $(SRC_DIR)/bootloader/initrix.asm -o $(BUILD_DIR)/initrix.bin

kernel: $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c always
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

# Ассемблерные файлы
$(BUILD_DIR)/entry32.o: $(SRC_DIR)/kernel32/kernel.asm always
	$(ASM) -f elf32 $< -o $@

$(BUILD_DIR)/sysenter_asm.o: $(SRC_DIR)/kernel32/sysenter/sysenter.asm always
	$(ASM) -f elf32 $< -o $@

$(BUILD_DIR)/interrupts_asm.o: $(SRC_DIR)/interrupts/interrupts.asm always
	$(ASM) -f elf32 $< -o $@

ASM_OBJECTS = $(BUILD_DIR)/entry32.o $(BUILD_DIR)/sysenter_asm.o $(BUILD_DIR)/interrupts_asm.o

$(BUILD_DIR)/kernel.bin: always $(ASM_OBJECTS) $(C_OBJECTS)
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/linker.ld -o $(BUILD_DIR)/kernel.elf $(ASM_OBJECTS) $(C_OBJECTS)
	objcopy -O binary $(BUILD_DIR)/kernel.elf $(BUILD_DIR)/kernel.bin


run: floppy
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy -serial stdio -machine pc

always:
	mkdir -p $(BUILD_DIR)

# Дебаг: запускает QEMU в режиме ожидания GDB
debug: floppy
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy -serial stdio -s -S -no-reboot -no-shutdown

disk: always
	@echo "[+] Creating disk"
	@dd if=/dev/zero of=$(DISK_IMG) bs=1M count=$(DISK_SIZE_MB) 2>/dev/null

	@echo "[+] Creating MBR at disk..."
	@printf "label: dos\nlabel-id: 0x12345678\ndevice: $(DISK_IMG)\nunit: sectors\n\n$(DISK_IMG)1 : start=2048, type=c\n" | sfdisk $(DISK_IMG) > /dev/null

	@echo "[+] Formatting disk on FAT32..."
	@mformat -i $(DISK_IMG)@@1048576 -F -h 64 -t 32 -n 64 ::

	@if [ -f $(INIT_BIN) ]; then \
		mcopy -o -i $(DISK_IMG)@@1048576 $(INIT_BIN) ::/INIT.BIN; \
		echo "[+] Disk successful created!"; \
	else \
		echo "[!] Warning: $(INIT_BIN) not found"; \
	fi

run-with-disk: floppy disk
	qemu-system-i386 \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-drive file=$(DISK_IMG),format=raw,index=0,media=disk \
		-boot order=a \
		-serial stdio \
		-machine pc 

run-with-disk-and-debug: floppy disk
	qemu-system-i386 \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-drive file=$(DISK_IMG),format=raw,index=0,media=disk \
		-boot order=a \
		-serial stdio \
		-machine pc \
		-s -S \
		-no-reboot \
		-no-shutdown


clean:
	rm -rf $(BUILD_DIR)
	rm -f $(SRC_DIR)/interrupts/*.o $(SRC_DIR)/libc/*/*.o