# © Realix > Makefile
# (22.03.26) v0.02
# ================

# Конфигурация
ASM = nasm
ASMFLAGS = -f bin -i $(SRC_DIR)
SRC_DIR = source
BUILD_DIR = build

.PHONY: all floppy bootix run clean always

# Запуск по умолчанию
all: floppy

# Сборка образа диска (floppy)
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix
	cp $(BUILD_DIR)/bootix.bin $(BUILD_DIR)/realix.img
	truncate -s 1440k $(BUILD_DIR)/realix.img

# Сборка загрузчика (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootix.asm -o $(BUILD_DIR)/bootix.bin

# Запуск собранного образа диска
run: floppy
	qemu-system-x86_64 -fda $(BUILD_DIR)/realix.img

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)

# Очистка
clean:
	rm -rf $(BUILD_DIR)/*