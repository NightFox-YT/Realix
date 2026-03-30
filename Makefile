# © Realix > Makefile
# (28.03.26) v0.04
# ================

# Конфигурация
ASM = nasm
ASMFLAGS = -f bin -i $(SRC_DIR)
SRC_DIR = source
BUILD_DIR = build

.PHONY: all floppy bootix initrix run clean always

# Запуск по умолчанию
all: floppy

# Сборка образа диска (floppy)
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix initrix
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mkfs.fat -F 12 -n "REALIX" $(BUILD_DIR)/realix.img
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/initrix.bin "::initrix.bin"

# Сборка загрузчика (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootix.asm -o $(BUILD_DIR)/bootix.bin

# Сборка инициализатора (bin)
initrix: $(BUILD_DIR)/initrix.bin

$(BUILD_DIR)/initrix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/initrix.asm -o $(BUILD_DIR)/initrix.bin

# Запуск собранного образа диска
run: floppy
	qemu-system-x86_64 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)

# Очистка
clean:
	rm -rf $(BUILD_DIR)/*