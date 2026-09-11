#!/usr/bin/env bash
# © Realix > Скрипт запуска в QEMU
# Пересобирает образ (если нужно) и запускает realix.img в qemu-system-x86_64.
#
# Использование:
#   ./run-qemu.sh              # обычный запуск
#   ./run-qemu.sh --rebuild    # принудительная пересборка (make clean && make)
#   ./run-qemu.sh --nova       # запуск с NovaAI моделью (требует KVM)
#   ./run-qemu.sh --nova-nokvm # запуск с NovaAI моделью без KVM (медленно)

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

IMAGE="build/realix.img"
MODE="normal"
REBUILD=0

for arg in "$@"; do
    case "$arg" in
        --rebuild)     REBUILD=1 ;;
        --nova)        MODE="nova" ;;
        --nova-nokvm)  MODE="nova-nokvm" ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Неизвестный аргумент: $arg" >&2
            exit 1
            ;;
    esac
done

if [ "$REBUILD" -eq 1 ]; then
    echo "[*] Пересборка проекта..."
    make clean
fi

if [ "$REBUILD" -eq 1 ] || [ ! -f "$IMAGE" ]; then
    echo "[*] Сборка образа..."
    make
fi

case "$MODE" in
    normal)
        echo "[*] Запуск QEMU..."
        exec qemu-system-x86_64 -display gtk -drive file="$IMAGE",format=raw,if=floppy
        ;;
    nova)
        MODEL="source/kernel32/nova-models/qwen_nova_q8.gguf"
        [ -f "$MODEL" ] || { echo "Не найдена модель: $MODEL" >&2; exit 1; }
        echo "[*] Запуск QEMU с NovaAI (KVM)..."
        exec qemu-system-x86_64 -display gtk -enable-kvm -cpu host -m 2G \
            -drive file="$IMAGE",format=raw,if=floppy \
            -device loader,file="$MODEL",addr=0x10000000
        ;;
    nova-nokvm)
        MODEL="source/kernel32/nova-models/qwen_nova_q8.gguf"
        [ -f "$MODEL" ] || { echo "Не найдена модель: $MODEL" >&2; exit 1; }
        echo "[*] Запуск QEMU с NovaAI (без KVM, медленно)..."
        exec qemu-system-x86_64 -display gtk -m 2G \
            -drive file="$IMAGE",format=raw,if=floppy \
            -device loader,file="$MODEL",addr=0x10000000
        ;;
esac
