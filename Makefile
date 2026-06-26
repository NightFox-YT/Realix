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


# Запуск собранного образа диска
run: floppy
	qemu-system-x86_64 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy

# Запуск с user-mode сетью (SLIRP) — ТОЛЬКО IP (TCP/UDP/ICMP), ARP не работает
# ARP-ответы обрабатываются внутри SLIRP и не пробрасываются на RTL8139
run-net: floppy
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy -netdev user,id=n0 -device rtl8139,netdev=n0

# Запуск с TAP-сетью — полный Ethernet (ARP работает)
# Требует sudo и предварительной настройки tap0:
#   sudo ip tuntap add tap0 mode tap user $(whoami)
#   sudo ip addr add 10.0.2.1/24 dev tap0
#   sudo ip link set tap0 up
run-tap: floppy
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-netdev tap,id=n0,ifname=tap0,script=no,downscript=no \
		-device rtl8139,netdev=n0

# Автоматическая настройка TAP-интерфейса и запуск (требует sudo)
run-tap-auto: floppy
	@echo "[*] Setting up tap0..."
	sudo ip tuntap add tap0 mode tap user $$(whoami) 2>/dev/null || true
	sudo ip addr add 10.0.2.1/24 dev tap0 2>/dev/null || true
	sudo ip link set tap0 up
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-netdev tap,id=n0,ifname=tap0,script=no,downscript=no \
		-device rtl8139,netdev=n0
	sudo ip link set tap0 down 2>/dev/null || true
	sudo ip tuntap del tap0 mode tap 2>/dev/null || true
	@echo "[*] tap0 cleaned up"

# Запуск с TAP + dnsmasq для DHCP и ARP-ответов (требует sudo)
# dnsmasq на tap0 будет раздавать IP из пула 10.0.2.100–10.0.2.200
# и отвечать на ARP-запросы к этим адресам и к 10.0.2.1 (шлюз).
# После выхода из QEMU dnsmasq и tap0 автоматически чистятся.
run-tap-dhcp: floppy
	@echo "[*] Setting up tap0 + dnsmasq..."
	sudo ip tuntap add tap0 mode tap user $$(whoami) 2>/dev/null || true
	sudo ip addr add 10.0.2.1/24 dev tap0 2>/dev/null || true
	sudo ip link set tap0 up
	sudo dnsmasq \
		--interface=tap0 \
		--dhcp-range=10.0.2.100,10.0.2.200,255.255.255.0,12h \
		--port=0 \
		--no-daemon \
		--pid-file=/tmp/dnsmasq-tap0.pid &
	sleep 1
	@echo "[*] dnsmasq started (DHCP pool: 10.0.2.100-200)"
	qemu-system-i386 -drive file=$(BUILD_DIR)/realix.img,format=raw,if=floppy \
		-netdev tap,id=n0,ifname=tap0,script=no,downscript=no \
		-device rtl8139,netdev=n0
	@echo "[*] Cleaning up..."
	sudo kill $$(cat /tmp/dnsmasq-tap0.pid 2>/dev/null) 2>/dev/null || true
	sudo rm -f /tmp/dnsmasq-tap0.pid
	sudo ip link set tap0 down 2>/dev/null || true
	sudo ip tuntap del tap0 mode tap 2>/dev/null || true
	@echo "[*] tap0 and dnsmasq cleaned up"

# Подготовка к сборке
always:
	mkdir -p $(BUILD_DIR)


# Очистка
clean:
	$(MAKE) -C $(SRC_DIR)/kernel16 BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	$(MAKE) -C $(SRC_DIR)/kernel32 BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	rm -rf $(BUILD_DIR)/*
