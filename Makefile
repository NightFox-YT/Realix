# © Realix > Makefile: Main (NASM)
# (21.06.26) v0.07
# ================

# Конфигурация
ASM = nasm
ASMFLAGS = -f bin -i $(SRC_DIR)
SRC_DIR = source
BUILD_DIR = build

.PHONY: all floppy bootix initrix kernel16 kernel32 clean always run

# Запуск по умолчанию
all: floppy


# Сборка образа диска (floppy)
floppy: $(BUILD_DIR)/realix.img

$(BUILD_DIR)/realix.img: bootix initrix kernel16 kernel32 always
	dd if=/dev/zero of=$(BUILD_DIR)/realix.img bs=512 count=2880
	mformat -i $(BUILD_DIR)/realix.img -f 1440 ::
	dd if=$(BUILD_DIR)/bootix.bin of=$(BUILD_DIR)/realix.img conv=notrunc
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/initrix.bin "::initrix.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/kernel16.bin "::kernel16.bin"
	mcopy -i $(BUILD_DIR)/realix.img $(BUILD_DIR)/kernel32.bin "::kernel32.bin"


# Сборка загрузчика (bin)
bootix: $(BUILD_DIR)/bootix.bin

$(BUILD_DIR)/bootix.bin:
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/bootix.asm -o $(BUILD_DIR)/bootix.bin


# Сборка инициализатора (bin)
initrix: $(BUILD_DIR)/initrix.bin

$(BUILD_DIR)/initrix.bin:
	$(ASM) $(ASMFLAGS) $(SRC_DIR)/bootloader/initrix.asm -o $(BUILD_DIR)/initrix.bin


# Сборка 16-битного ядра (NASM + C в будущем)
kernel16: always
	$(MAKE) -C $(SRC_DIR)/kernel16 BUILD_DIR=$(abspath $(BUILD_DIR)) SRC_DIR=$(abspath $(SRC_DIR))


# Сборка 32-битного ядра (Rust)
kernel32: always
	$(MAKE) -C $(SRC_DIR)/kernel32 BUILD_DIR=$(abspath $(BUILD_DIR))


# Запуск с KVM (быстро), без NOVA
run: floppy
	qemu-system-x86_64 -enable-kvm -cpu host -m 2G \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy

# Запуск NOVA с KVM + GPT-2 GGUF
run-nova: floppy
	qemu-system-x86_64 -enable-kvm -cpu host -m 2G \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-device loader,file=$(SRC_DIR)/kernel32/build/nova_f16.gguf,addr=0x10000000

# Запуск NOVA с Qwen 2.5 0.5B (1.6 GB)
run-qwen: floppy
	qemu-system-x86_64 -enable-kvm -cpu host -m 2G \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-device loader,file=$(SRC_DIR)/kernel32/build/qwen_nova_q8.gguf,addr=0x10000000

# Запуск NOVA без KVM (медленно, отладка)
run-nova-nokvm: floppy
	qemu-system-x86_64 -m 2G \
		-drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-device loader,file=$(SRC_DIR)/kernel32/build/nova_f16.gguf,addr=0x10000000

# Запуск NOVA с KVM (старая цель, алиас на run-nova)
run-nova-kvm: run-nova
	@true


nova_f16.gguf: $(SRC_DIR)/kernel32/build/nova_f16.gguf
	@true


# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)


# Очистка
clean:
	$(MAKE) -C $(SRC_DIR)/kernel16 BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	$(MAKE) -C $(SRC_DIR)/kernel32 BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	rm -rf $(BUILD_DIR)/*