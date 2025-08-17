# Realix > Build Script
# (C) v0.02 | 17.08.25
# ===================

# Конфигурация
ASM = nasm
SRC_DIR = source
BUILD_DIR = build

.PHONY: all floppy bootix clean always

# Запуск по умолчанию
all: floppy

# Создание образа диска (floppy)
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix
	cp $(BUILD_DIR)/bootix.bin $(BUILD_DIR)/realix.img
	truncate -s 1440k $(BUILD_DIR)/realix.img

# Сборка Bootix (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: always
	$(ASM) $(SRC_DIR)/bootix.asm -f bin -o $(BUILD_DIR)/bootix.bin -i $(SRC_DIR)/kernel

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)

# Очистка
clean:
	rm -rf $(BUILD_DIR)/*