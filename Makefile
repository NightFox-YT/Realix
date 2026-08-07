# © Realix > Makefile: Main
# (27.07.26) v0.1
# ================

# Конфигурация
SRC_DIR   := source
BUILD_DIR := build

# Каталоги подпроектов
BOOTLOADER_DIR := $(SRC_DIR)/bootloader
KERNEL16_DIR   := $(SRC_DIR)/kernel16
KERNEL32_DIR   := $(SRC_DIR)/kernel32

# Параметры, передаваемые в дочерние Makefile
SUBMAKE_VARS := BUILD_DIR=$(abspath $(BUILD_DIR)) SRC_DIR=$(abspath $(SRC_DIR))

# Образ диска и его содержимое
IMAGE        := $(BUILD_DIR)/realix.img
ISO          := $(BUILD_DIR)/realix.iso
BOOTIX_BIN   := $(BUILD_DIR)/bootix.bin
INITRIX_BIN  := $(BUILD_DIR)/initrix.bin
KERNEL16_BIN := $(BUILD_DIR)/kernel16.bin
KERNEL32_BIN := $(BUILD_DIR)/kernel32.bin
APP16_RLX    := $(BUILD_DIR)/app16.rlx
APP32_RLX    := $(BUILD_DIR)/app32.rlx
SNAKE32_RLX  := $(BUILD_DIR)/snake32.rlx
RLXFETCH_RLX  := $(BUILD_DIR)/rlxfetch.rlx
RLXFETCH16_RLX:= $(BUILD_DIR)/rlxfetch16.rlx
EXEC16_RLX    := $(BUILD_DIR)/exec16.rlx
DESKTOP32_RLX := $(BUILD_DIR)/desktop32.rlx
CALC_RLX      := $(BUILD_DIR)/calc.rlx

.PHONY: all floppy iso bootloader bootix initrix kernel16 kernel32 apps run clean

# Запуск по умолчанию
all: floppy


# Сборка образа диска (floppy 1.44 МБ)
floppy: $(IMAGE)

$(IMAGE): bootloader apps kernel16 kernel32 | $(BUILD_DIR)
	dd if=/dev/zero of=$(IMAGE) bs=512 count=2880
	mformat -i $(IMAGE) -f 1440 ::
	dd if=$(BOOTIX_BIN) of=$(IMAGE) conv=notrunc
	mcopy -i $(IMAGE) $(INITRIX_BIN) "::initrix.bin"
	mcopy -i $(IMAGE) $(KERNEL16_BIN) "::kernel16.bin"
	mcopy -i $(IMAGE) $(KERNEL32_BIN) "::kernel32.bin"
	mcopy -i $(IMAGE) $(APP16_RLX) "::app16.rlx"
	mcopy -i $(IMAGE) $(APP32_RLX) "::app32.rlx"
	mcopy -i $(IMAGE) $(SNAKE32_RLX) "::snake32.rlx"
	mcopy -i $(IMAGE) $(RLXFETCH_RLX) "::rlxfetch.rlx"
	mcopy -i $(IMAGE) $(RLXFETCH16_RLX) "::rlxfetch16.rlx"
	mcopy -i $(IMAGE) $(EXEC16_RLX) "::exec16.rlx"
	mcopy -i $(IMAGE) $(DESKTOP32_RLX) "::desktop32.rlx"
	mcopy -i $(IMAGE) $(CALC_RLX) "::calc.rlx"

# Сборка демо-приложений .RLX
apps: $(APP16_RLX) $(APP32_RLX) $(SNAKE32_RLX) $(RLXFETCH_RLX) $(RLXFETCH16_RLX) $(EXEC16_RLX) $(DESKTOP32_RLX) $(CALC_RLX)

$(APP16_RLX): $(SRC_DIR)/apps/app16.asm | $(BUILD_DIR)
	nasm -f bin -i $(SRC_DIR) $(SRC_DIR)/apps/app16.asm -o $@

$(APP32_RLX): $(SRC_DIR)/apps/app32.asm | $(BUILD_DIR)
	nasm -f bin -i $(SRC_DIR) $(SRC_DIR)/apps/app32.asm -o $@

$(SNAKE32_RLX): $(SRC_DIR)/apps/snake32.asm | $(BUILD_DIR)
	nasm -f bin -i $(SRC_DIR) $(SRC_DIR)/apps/snake32.asm -o $@

$(RLXFETCH_RLX): $(SRC_DIR)/apps/rlxfetch.c $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/linker32.ld | $(BUILD_DIR)
	gcc -m32 -ffreestanding -nostdlib -fno-pic -fno-pie -Wl,--build-id=none -T $(SRC_DIR)/apps/linker32.ld $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/rlxfetch.c -o $@ -Wl,--oformat=binary

$(RLXFETCH16_RLX): $(SRC_DIR)/apps/rlxfetch16.asm | $(BUILD_DIR)
	nasm -f bin -i $(SRC_DIR) $(SRC_DIR)/apps/rlxfetch16.asm -o $@

$(EXEC16_RLX): $(SRC_DIR)/apps/exec16.c $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/linker32.ld | $(BUILD_DIR)
	gcc -m32 -ffreestanding -nostdlib -fno-pic -fno-pie -Wl,--build-id=none -T $(SRC_DIR)/apps/linker32.ld $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/exec16.c -o $@ -Wl,--oformat=binary

$(DESKTOP32_RLX): $(SRC_DIR)/apps/desktop32.c $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/linker32.ld | $(BUILD_DIR)
	gcc -m32 -ffreestanding -nostdlib -fno-pic -fno-pie -Wl,--build-id=none -T $(SRC_DIR)/apps/linker32.ld $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/desktop32.c -o $@ -Wl,--oformat=binary

$(CALC_RLX): $(SRC_DIR)/apps/calc.c $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/linker32.ld | $(BUILD_DIR)
	gcc -m32 -ffreestanding -nostdlib -fno-pic -fno-pie -Wl,--build-id=none -T $(SRC_DIR)/apps/linker32.ld $(SRC_DIR)/apps/entry32.s $(SRC_DIR)/apps/calc.c -o $@ -Wl,--oformat=binary

# Сборка ISO-образа
iso: $(ISO)

$(ISO): $(IMAGE)
	xorriso -as mkisofs -V "REALIX_OS" -b realix.img -c boot.cat -m "realix.iso" -o $(ISO) $(BUILD_DIR)


# Сборка загрузчика: Bootix (Stage 1) + Initrix (Stage 2)
bootloader:
	$(MAKE) -C $(BOOTLOADER_DIR) $(SUBMAKE_VARS)

# Сборка отдельных стадий загрузчика
bootix initrix:
	$(MAKE) -C $(BOOTLOADER_DIR) $(SUBMAKE_VARS) $@


# Сборка 16-битного ядра (NASM + C в будущем)
kernel16:
	$(MAKE) -C $(KERNEL16_DIR) $(SUBMAKE_VARS)


# Сборка 32-битного ядра (Rust)
kernel32:
	$(MAKE) -C $(KERNEL32_DIR) BUILD_DIR=$(abspath $(BUILD_DIR))


# Запуск собранного образа диска
run: floppy
	qemu-system-x86_64 -display sdl -drive file=$(IMAGE),format=raw,if=floppy

# Запуск ISO-образа в QEMU
run-iso: iso
	qemu-system-x86_64 -display sdl -cdrom $(ISO)


# Очистка
clean:
	$(MAKE) -C $(BOOTLOADER_DIR) $(SUBMAKE_VARS) clean
	$(MAKE) -C $(KERNEL16_DIR) $(SUBMAKE_VARS) clean
	$(MAKE) -C $(KERNEL32_DIR) BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	rm -rf $(BUILD_DIR)
