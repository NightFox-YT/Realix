# © Realix > Makefile
# (18.06.26) v0.05
# ================

# Конфигурация инструментов
ASM = nasm
CC = gcc
LD = ld

# Направляем компилятор на наши собственные заголовочные файлы libc и include
INCLUDES = -I./source/libc -I./source/include -I./source/drivers-global

# Флаги компиляции для x86
CFLAGS = -m32 -ffreestanding -O2 -Wall -Wextra -fno-exceptions -fno-builtin -fno-pic -fno-pie -nostdlib $(INCLUDES)
ASMFLAGS = -f bin -i $(SRC_DIR)/
ASM_OBJ_FLAGS = -f elf32 -i $(SRC_DIR)/

SRC_DIR = source
BUILD_DIR = build

# Автоматический поиск всех Си-файлов в проекте
C_SOURCES = $(wildcard kernel.c) \
            $(wildcard $(SRC_DIR)/libc/atomic/*.c) \
            $(wildcard $(SRC_DIR)/libc/io/*.c) \
            $(wildcard $(SRC_DIR)/libc/string/*.c) \
            $(wildcard $(SRC_DIR)/drivers-global/serial/*.c) \
            $(wildcard $(SRC_DIR)/interrupts/*.c)

C_OBJECTS = $(patsubst %.c, $(BUILD_DIR)/%.o, $(notdir $(C_SOURCES)))

.PHONY: all floppy bootix initrix kernel run clean always

# Запуск по умолчанию
all: floppy

# Сборка образа диска
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix initrix kernel
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mformat -i $(BUILD_DIR)/realix.img -f 1440 ::
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/initrix.bin "::initrix.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/kernel.bin "::kernel.bin"

# Сборка загрузчика (MBR, 512 байт бинарник)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/bootix.asm -o $(BUILD_DIR)/bootix.bin

# Сборка инициализатора
initrix: $(BUILD_DIR)/initrix.bin

$(BUILD_DIR)/initrix.bin: always
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/initrix.asm -o $(BUILD_DIR)/initrix.bin

kernel: $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/%.o: %.c always
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/libc/atomic/%.c always
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/libc/io/%.c always
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/libc/string/%.c always
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/drivers-global/serial/%.c always
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/interrupts/%.c always
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/kernel.bin: always $(C_OBJECTS)
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/kernel16/kernel.asm -o $(BUILD_DIR)/kernel.bin

# Запуск собранного образа диска
run: floppy
	qemu-system-x86_64 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy -serial stdio

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)

# Очистка
clean:
	rm -rf $(BUILD_DIR)