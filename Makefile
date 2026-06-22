# © Realix > Makefile
# (28.03.26) v0.04
# ================

# Конфигурация
ASM = nasm
ASMFLAGS = -f bin -i $(SRC_DIR)
SRC_DIR = source
BUILD_DIR = build

# Параметры сборки хранилища Vault (переопределяются при вызове make)
VAULT_PASSWORD ?= changeme
VAULT_SECRET   ?= Realix vault: replace this secret at build time.
VAULT_ITERS    ?= 100000

.PHONY: all floppy bootix initrix kernel vault run clean always

# Запуск по умолчанию
all: floppy


# Сборка образа диска (floppy)
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix initrix kernel vault
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mformat -i $(BUILD_DIR)/realix.img -f 1440 ::
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/initrix.bin "::initrix.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/kernel.bin "::kernel.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/VAULT.BIN "::VAULT.BIN"


# Сборка загрузчика (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/bootix.asm -o $(BUILD_DIR)/bootix.bin


# Сборка инициализатора (bin)
initrix: $(BUILD_DIR)/initrix.bin

$(BUILD_DIR)/initrix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/initrix.asm -o $(BUILD_DIR)/initrix.bin


# Сборка ядра (bin)
kernel: $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/kernel.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/kernel16/kernel.asm -o $(BUILD_DIR)/kernel.bin


# Сборка хранилища Vault (VAULT.BIN, генерируется vault_forge.py)
vault: $(BUILD_DIR)/VAULT.BIN

$(BUILD_DIR)/VAULT.BIN: always
	python3 tools/vault_forge.py forge --password "$(VAULT_PASSWORD)" --text "$(VAULT_SECRET)" --iters $(VAULT_ITERS) --out $(BUILD_DIR)/VAULT.BIN


# Запуск собранного образа диска
run: floppy
	qemu-system-x86_64 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy


# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)


# Очистка
clean:
	rm -rf $(BUILD_DIR)/*