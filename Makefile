# Realix > Build Script
# (C) v0.03 | 19.08.25
# =====================

# Конфигурация
ASM = nasm
SRC_DIR = source
BUILD_DIR = build

.PHONY: all floppy bootix clean always

# Запуск по умолчанию
all: floppy

# Сборка образа диска (floppy)
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mkfs.fat -F 12 -n "Realix" $(BUILD_DIR)/realix.img
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc

# Сборка загрузчика (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: always
	$(ASM) $(SRC_DIR)/bootix.asm -f bin -o $(BUILD_DIR)/bootix.bin -i $(SRC_DIR)/kernel -i $(SRC_DIR)/disk -i $(SRC_DIR)/fat

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)

# Очистка
clean:
	rm -rf $(BUILD_DIR)/*