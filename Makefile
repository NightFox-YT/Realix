# © Realix > Makefile: Main
# (27.07.26) v0.1
# ================

# Конфигурация
SRC_DIR   := source
BUILD_DIR := build
APPS_DIR  := apps

# Каталоги подпроектов
BOOTLOADER_DIR := $(SRC_DIR)/bootloader
KERNEL16_DIR   := $(SRC_DIR)/kernel16
KERNEL32_DIR   := $(SRC_DIR)/kernel32

# Параметры, передаваемые в дочерние Makefile
SUBMAKE_VARS := BUILD_DIR=$(abspath $(BUILD_DIR)) SRC_DIR=$(abspath $(SRC_DIR))

# Образ диска и его содержимое
IMAGE        := $(BUILD_DIR)/realix.img
BOOTIX_BIN   := $(BUILD_DIR)/bootix.bin
INITRIX_BIN  := $(BUILD_DIR)/initrix.bin
KERNEL16_BIN := $(BUILD_DIR)/kernel16.bin
KERNEL32_BIN := $(BUILD_DIR)/kernel32.bin
APPS_SRC     := $(shell find $(APPS_DIR) -name '*.asm')
APPS_OBJS    := $(patsubst $(APPS_DIR)/%.asm,$(BUILD_DIR)/%.rlx,$(APPS_SRC))

# Компилятор и флаги для приложений на NASM
ASM      := nasm
ASMFLAGS := -f bin -i $(SRC_DIR)

.PHONY: all floppy bootloader bootix initrix kernel16 kernel32 apps16 run clean

# Запуск по умолчанию
all: floppy


# Сборка образа диска (floppy 1.44 МБ)
floppy: $(IMAGE)

$(IMAGE): bootloader kernel16 kernel32 apps16
	dd if=/dev/zero of=$(IMAGE) bs=512 count=2880
	mformat -i $(IMAGE) -f 1440 ::
	dd if=$(BOOTIX_BIN) of=$(IMAGE) conv=notrunc
	mcopy -i $(IMAGE) $(INITRIX_BIN) "::initrix.bin"
	mcopy -i $(IMAGE) $(KERNEL16_BIN) "::kernel16.bin"
	mcopy -i $(IMAGE) $(KERNEL32_BIN) "::kernel32.bin"
	mcopy -i $(IMAGE) $(APPS_OBJS) "::"


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
	$(MAKE) -C $(KERNEL32_DIR) $(SUBMAKE_VARS)


# Сборка 16-битных пользовательских приложений
apps16: $(APPS_OBJS)

$(BUILD_DIR)/%.rlx: $(APPS_DIR)/%.asm
	$(ASM) $(ASMFLAGS) $< -o $@


# Запуск (без NOVA)
run: floppy
	qemu-system-x86_64 -drive file=$(IMAGE),format=raw,if=floppy

# Запуск (NOVA Qwen 2.5 0.5B (1.6 GB) с KVM)
run-nova: floppy
	qemu-system-x86_64 -enable-kvm -cpu host -m 2G \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-device loader,file=$(SRC_DIR)/kernel32/nova-models/qwen_nova_q8.gguf,addr=0x10000000

# Запуск (NOVA Qwen 2.5 0.5B (1.6 GB) без KVM - медленно)
run-nova-nokvm: floppy
	qemu-system-x86_64 -m 2G \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-device loader,file=$(SRC_DIR)/kernel32/nova-models/qwen_nova_q8.gguf,addr=0x10000000

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)


# Очистка
clean:
	$(MAKE) -C $(BOOTLOADER_DIR) $(SUBMAKE_VARS) clean
	$(MAKE) -C $(KERNEL16_DIR) $(SUBMAKE_VARS) clean
	$(MAKE) -C $(KERNEL32_DIR) BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	rm -rf $(BUILD_DIR)
