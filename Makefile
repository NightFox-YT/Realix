# © Realix > Makefile
# (23.06.26) v0.05 - Модульная синхронизация
# ==========================================

# Конфигурация инструментов
ASM = nasm
CC = gcc
LD = ld
PY = python3
DISK_IMG = realix_disk_fat32.img
INIT_BIN = $(BUILD_DIR)/initrix.bin
DISK_SIZE_MB = 64
INCLUDES = -I./source/libc/klibc -I./source/include

CFLAGS = -m32 -g -ffreestanding -O2 -Wall -Wextra -fno-exceptions -D__realix__ \
         -fno-builtin -fno-pic -fno-pie -fno-stack-protector -nostdlib $(INCLUDES)

SRC_DIR = source
BUILD_DIR = build
DRIVERS_BUILD_DIR = $(BUILD_DIR)/modules

# 1. Код ядра
KERNEL_SOURCES = $(wildcard $(SRC_DIR)/kernel32/*.c) \
                 $(wildcard $(SRC_DIR)/kernel32/memory/*.c) \
                 $(wildcard $(SRC_DIR)/kernel32/sysenter/*.c) \
                 $(wildcard $(SRC_DIR)/drivers-32/serial/*.c) \
                 $(wildcard $(SRC_DIR)/interrupts/*.c) \
                 $(wildcard $(SRC_DIR)/init/*.c) \
				 $(wildcard $(SRC_DIR)/libc/klibc/*.c) \
				 $(wildcard $(SRC_DIR)/vfs/*.c) \
				 $(wildcard $(SRC_DIR)/libc/klibc/atomic/*.c) \
				 $(wildcard $(SRC_DIR)/libc/klibc/io/*.c) \
				 $(wildcard $(SRC_DIR)/libc/klibc/stdlib/*.c) \
				 $(wildcard $(SRC_DIR)/libc/klibc/string/*.c) 

KERNEL_OBJECTS = $(patsubst $(SRC_DIR)/%.c, $(BUILD_DIR)/%.o, $(KERNEL_SOURCES))
ASM_OBJECTS = $(BUILD_DIR)/entry32.o $(BUILD_DIR)/sysenter_asm.o $(BUILD_DIR)/interrupts_asm.o

# 2. Список модулей
MODULES = $(DRIVERS_BUILD_DIR)/ata.rcom \
          $(DRIVERS_BUILD_DIR)/pci.rcom \
          $(DRIVERS_BUILD_DIR)/fat32.rcom \
          $(DRIVERS_BUILD_DIR)/keyboard.rcom \
          $(DRIVERS_BUILD_DIR)/rxbdph.rcom 

.PHONY: all floppy bootix initrix kernel modules ramfs run clean always debug disk run-with-disk run-with-disk-and-debug help

all: floppy

floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix initrix kernel ramfs
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mformat -i $(BUILD_DIR)/realix.img -f 1440 ::
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/initrix.bin "::initrix.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/kernel.bin "::kernel.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/ramfs.img "::ramfs.img"

# Правила для 16-битных компонентов загрузки
bootix: always
	$(ASM) -i source/ -f bin $(SRC_DIR)/bootloader/bootix.asm -o $(BUILD_DIR)/bootix.bin

initrix: always
	$(ASM) -i source/ -f bin $(SRC_DIR)/bootloader/initrix.asm -o $(BUILD_DIR)/initrix.bin

# Сборка ассемблерных файлов ядра
$(BUILD_DIR)/entry32.o: $(SRC_DIR)/kernel32/kernel.asm always
	$(ASM) -f elf32 $< -o $@

$(BUILD_DIR)/sysenter_asm.o: $(SRC_DIR)/kernel32/sysenter/sysenter.asm always
	$(ASM) -f elf32 $< -o $@

$(BUILD_DIR)/interrupts_asm.o: $(SRC_DIR)/interrupts/interrupts.asm always
	$(ASM) -f elf32 $< -o $@

kernel: $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/kernel.bin: always $(ASM_OBJECTS) $(KERNEL_OBJECTS)
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/linker.ld -o $(BUILD_DIR)/kernel.elf $(ASM_OBJECTS) $(KERNEL_OBJECTS)
	objcopy -O binary $(BUILD_DIR)/kernel.elf $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c always
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

# 3. Правила сборки .rcom

$(DRIVERS_BUILD_DIR)/ata.rcom: $(SRC_DIR)/drivers-32/ata/ata.c always
	@mkdir -p $(DRIVERS_BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(DRIVERS_BUILD_DIR)/ata.o
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/driver.ld -o $(DRIVERS_BUILD_DIR)/ata.bin $(DRIVERS_BUILD_DIR)/ata.o
	objcopy -O binary $(DRIVERS_BUILD_DIR)/ata.bin $(DRIVERS_BUILD_DIR)/ata.raw
	$(PY) utils/make_rcom.py $(DRIVERS_BUILD_DIR)/ata.raw $@ ata_driver

$(DRIVERS_BUILD_DIR)/pci.rcom: $(SRC_DIR)/drivers-32/pci/pci.c always
	@mkdir -p $(DRIVERS_BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(DRIVERS_BUILD_DIR)/pci.o
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/driver.ld -o $(DRIVERS_BUILD_DIR)/pci.bin $(DRIVERS_BUILD_DIR)/pci.o
	objcopy -O binary $(DRIVERS_BUILD_DIR)/pci.bin $(DRIVERS_BUILD_DIR)/pci.raw
	$(PY) utils/make_rcom.py $(DRIVERS_BUILD_DIR)/pci.raw $@ pci_driver

$(DRIVERS_BUILD_DIR)/rxbdph.rcom: $(SRC_DIR)/drivers-32/rxbdph/rxbdph.c always
	@mkdir -p $(DRIVERS_BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(DRIVERS_BUILD_DIR)/rxbdph.o
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/driver.ld -o $(DRIVERS_BUILD_DIR)/rxbdph.bin $(DRIVERS_BUILD_DIR)/rxbdph.o
	objcopy -O binary $(DRIVERS_BUILD_DIR)/rxbdph.bin $(DRIVERS_BUILD_DIR)/rxbdph.raw
	$(PY) utils/make_rcom.py $(DRIVERS_BUILD_DIR)/rxbdph.raw $@ rxbdph_driver

$(DRIVERS_BUILD_DIR)/keyboard.rcom: $(SRC_DIR)/drivers-32/keyboard/keyboard.c always
	@mkdir -p $(DRIVERS_BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(DRIVERS_BUILD_DIR)/keyboard.o
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/driver.ld -o $(DRIVERS_BUILD_DIR)/keyboard.bin $(DRIVERS_BUILD_DIR)/keyboard.o
	objcopy -O binary $(DRIVERS_BUILD_DIR)/keyboard.bin $(DRIVERS_BUILD_DIR)/keyboard.raw
	$(PY) utils/make_rcom.py $(DRIVERS_BUILD_DIR)/keyboard.raw $@ keyboard_driver

$(DRIVERS_BUILD_DIR)/fat32.rcom: $(SRC_DIR)/drivers-32/fat32/fat32.c always
	@mkdir -p $(DRIVERS_BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $(DRIVERS_BUILD_DIR)/fat32.o
	$(LD) -m elf_i386 -T $(SRC_DIR)/link/driver.ld -o $(DRIVERS_BUILD_DIR)/fat32.bin $(DRIVERS_BUILD_DIR)/fat32.o
	objcopy -O binary $(DRIVERS_BUILD_DIR)/fat32.bin $(DRIVERS_BUILD_DIR)/fat32.raw
	$(PY) utils/make_rcom.py $(DRIVERS_BUILD_DIR)/fat32.raw $@ fat32_driver

modules: $(MODULES)

# 4. Создание образа ramfs
ramfs: modules
	$(PY) utils/enjoy_rcoms.py $(BUILD_DIR)/ramfs.img $(MODULES)

run: floppy
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy -serial stdio -machine pc

always:
	mkdir -p $(BUILD_DIR)
	mkdir -p $(DRIVERS_BUILD_DIR)

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
	rm -f $(DISK_IMG)

help: 
	@printf "\n\033[36mOptions\033[0m:\n"
	@printf "\tmake\t\t\t\t- build realix\n"
	@printf "\tmake modules\t\t\t- build modules\n"
	@printf "\tmake run\t\t\t- run realix\n"
	@printf "\tmake always\t\t\t- create directory for build\n"
	@printf "\tmake debug\t\t\t- run and suspend QEMU for debug\n"
	@printf "\tmake disk\t\t\t- make MBR FAT32 disk\n"
	@printf "\tmake run-with-disk\t\t- run realix with created disk\n"
	@printf "\tmake run-with-disk-and-debug\t- run realix with disk and suspend to debug\n"
	@printf "\tmake clean\t\t\t- clean environment\n"
	@printf "\tmake help\t\t\t- show this help\n"
	@printf "The realix Makefile, (c) Realix.\n\n"