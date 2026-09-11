# © Realix > Makefile: Main
# (27.07.26) v0.1
# ================

# Конфигурация
SRC_DIR   := source
BUILD_DIR := build
APPS_DIR  := apps

# Каталоги подпроектов
BOOTLOADER_DIR       := $(SRC_DIR)/bootloader
KERNEL16_DIR         := $(SRC_DIR)/kernel16
KERNEL16_NIGHTLY_DIR := $(SRC_DIR)/kernel16-nightly
KERNEL32_DIR         := $(SRC_DIR)/kernel32
KERNEL32_NIGHTLY_DIR := $(SRC_DIR)/kernel32-nightly

# Параметры, передаваемые в дочерние Makefile
SUBMAKE_VARS := BUILD_DIR=$(abspath $(BUILD_DIR)) SRC_DIR=$(abspath $(SRC_DIR))

# Образ диска и его содержимое
IMAGE          := $(BUILD_DIR)/realix.img
BOOTIX_BIN     := $(BUILD_DIR)/bootix.bin
INITRIX_BIN    := $(BUILD_DIR)/initrix.bin
KERNEL16_BIN   := $(BUILD_DIR)/kernel16.bin
NIGHT16_BIN    := $(BUILD_DIR)/night16.bin
KERNEL32_BIN   := $(BUILD_DIR)/kernel32.bin
NIGHTLY_BIN    := $(BUILD_DIR)/nightly.bin
APPS_SRC       := $(shell find $(APPS_DIR) -name '*.asm')
APPS_OBJS      := $(patsubst $(APPS_DIR)/%.asm,$(BUILD_DIR)/%.rlx,$(APPS_SRC))
DOSAPPS_DIR    := dosapps
DOSAPPS_SRC    := $(shell find $(DOSAPPS_DIR) -name '*.asm' 2>/dev/null)
DOSAPPS_OBJS   := $(patsubst $(DOSAPPS_DIR)/%.asm,$(BUILD_DIR)/%.com,$(DOSAPPS_SRC))
DOSAPPS_BIN    := $(shell find $(DOSAPPS_DIR) -name '*.com' 2>/dev/null)

# Компилятор и флаги для приложений на NASM
ASM      := nasm
ASMFLAGS := -f bin -i $(SRC_DIR)

.PHONY: all floppy bootloader bootix initrix kernel16 kernel16-nightly kernel32 kernel32-nightly apps16 dosapps run clean

# Запуск по умолчанию
all: floppy


# Сборка образа диска (floppy 1.44 МБ)
floppy: $(IMAGE)

$(IMAGE): bootloader kernel16 kernel16-nightly kernel32 kernel32-nightly apps16 dosapps
	dd if=/dev/zero of=$(IMAGE) bs=512 count=2880
	mformat -i $(IMAGE) -f 1440 ::
	dd if=$(BOOTIX_BIN) of=$(IMAGE) conv=notrunc
	mcopy -i $(IMAGE) $(INITRIX_BIN) "::initrix.bin"
	mcopy -i $(IMAGE) $(KERNEL16_BIN) "::kernel16.bin"
	mcopy -i $(IMAGE) $(NIGHT16_BIN) "::night16.bin"
	mcopy -i $(IMAGE) $(KERNEL32_BIN) "::kernel32.bin"
	mcopy -i $(IMAGE) $(NIGHTLY_BIN) "::nightly.bin"
	mcopy -i $(IMAGE) $(APPS_OBJS) "::"
	$(if $(DOSAPPS_OBJS),mcopy -i $(IMAGE) $(DOSAPPS_OBJS) "::",)
	$(if $(DOSAPPS_BIN),mcopy -i $(IMAGE) $(DOSAPPS_BIN) "::",)


# Сборка загрузчика: Bootix (Stage 1) + Initrix (Stage 2)
bootloader:
	$(MAKE) -C $(BOOTLOADER_DIR) $(SUBMAKE_VARS)

# Сборка отдельных стадий загрузчика
bootix initrix:
	$(MAKE) -C $(BOOTLOADER_DIR) $(SUBMAKE_VARS) $@


# Сборка 16-битного ядра (NASM + C в будущем, стабильная ветка)
kernel16:
	$(MAKE) -C $(KERNEL16_DIR) $(SUBMAKE_VARS)

# Сборка 16-битного ядра Nightly (NASM, экспериментальная ветка - MS-DOS .COM приложения)
kernel16-nightly:
	$(MAKE) -C $(KERNEL16_NIGHTLY_DIR) $(SUBMAKE_VARS)


# Сборка 32-битного ядра (Rust, стабильная ветка)
kernel32:
	$(MAKE) -C $(KERNEL32_DIR) $(SUBMAKE_VARS)

# Сборка 32-битного ядра Nightly (Rust, экспериментальная ветка - FAT12/floppy/RLX32/...)
kernel32-nightly:
	$(MAKE) -C $(KERNEL32_NIGHTLY_DIR) $(SUBMAKE_VARS)


# Сборка 16-битных пользовательских приложений
apps16: $(APPS_OBJS)

$(BUILD_DIR)/%.rlx: $(APPS_DIR)/%.asm
	$(ASM) $(ASMFLAGS) $< -o $@

# Сборка тестовых MS-DOS .COM приложений (dosapps/*.asm) - пусто, если каталога нет
dosapps: $(DOSAPPS_OBJS)

$(BUILD_DIR)/%.com: $(DOSAPPS_DIR)/%.asm
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
	$(MAKE) -C $(KERNEL16_NIGHTLY_DIR) $(SUBMAKE_VARS) clean
	$(MAKE) -C $(KERNEL32_DIR) BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	$(MAKE) -C $(KERNEL32_NIGHTLY_DIR) BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	rm -rf $(BUILD_DIR)
