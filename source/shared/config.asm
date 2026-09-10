; © Realix > Config
; (15.08.26) v0.12
; ================

; Защита от повторного включения
%ifndef CONFIG_ASM
%define CONFIG_ASM


; ! Не забывайте менять эту строку
%define OS_VERSION 'v0.12'

; Управляющие символы и ASCII коды
%define ENTER 0x0D, 0x0A

; Системные адреса памяти
PCINFO_ADDR          equ 0x4500
FAT_BUFFER_ADDR      equ 0x0500
BOOTIX_LOAD_OFFSET   equ 0x7C00
INITRIX_LOAD_SEGMENT equ 0x07E0
INITRIX_LOAD_OFFSET  equ 0
KERNEL_LOAD_SEGMENT  equ 0x1000
KERNEL_LOAD_OFFSET   equ 0
KERNEL32_PHYS_ADDR   equ (KERNEL_LOAD_SEGMENT*16 + KERNEL_LOAD_OFFSET)

; Раскладка структуры PCINFO
PCINFO_ALL_MEM      equ 0   ; Размер всей памяти (МБ)
PCINFO_LOW_MEM      equ 4   ; Размер "нижней" памяти (КБ)
PCINFO_MMAP_ENTRIES equ 6   ; Кол-во записей карты памяти
PCINFO_DRIVE        equ 8   ; Номер загрузочного диска
PCINFO_VIDEOMODE    equ 10  ; Номер видеорежима (0 - текстовый, 1 - видеорежим)
PCINFO_MMAP         equ 12  ; Массив записей E820

; Поля VBE (см. bios-api/vbe.asm) - ДОБАВЛЕНЫ ПОСЛЕ PCINFO_MMAP (не между
; существующими полями), чтобы не сдвинуть уже используемые смещения.
; ❗️ Литералы, а не выражение через PCINFO_MMAP+... - build.rs (генератор
; config.rs для Rust) распознаёт только "ИМЯ equ ЧИСЛО", выражения молча
; пропускает (см. её же комментарии) - но значение то же самое:
; PCINFO_MMAP(12) + E820_MAX_ENTRIES(64)*E820_ENTRY_SIZE(24) = 1548
; PCINFO_VIDEO_WIDTH = 0 означает "обычный VGA mode 13h (320x200,
; 0xA0000)" - только ненулевое значение означает, что ниже заполнены
; настоящие данные VBE-режима
PCINFO_VIDEO_WIDTH  equ 1548  ; 16: Ширина экрана (px)
PCINFO_VIDEO_HEIGHT equ 1550  ; 16: Высота экрана (px)
PCINFO_VIDEO_STRIDE equ 1552  ; 16: Байт на строку (BytesPerScanLine)
PCINFO_VIDEO_LFB    equ 1554  ; 32: Физический адрес линейного фреймбуфера

; Маркеры кластеров FAT12 (общие для bootix и bios-api/fat12)
CHAIN_END   equ 0x0FF8  ; Кластер >= этого - конец цепочки (EOF)
BAD_CLUSTER equ 0x0FF7  ; Дефектный кластер

; Видеорежимы BIOS: ah - Установка режима, al - Выбранный режим
TEXT_MODE_80x25    equ 0x0003
VIDEO_MODE_320x200 equ 0x0013

; Видеорежим VBE (см. bios-api/vbe.asm) - 640x480x256. Не 0x100 (640x400,
; который был бы ровно 2x 320x200 без чёрной полосы) - тот режим не входит
; в "базовый" список, который гарантированно поддерживают все VBE BIOS
; (VirtualBox/QEMU/Bochs включительно), а 0x101 входит. Ширина всё равно
; масштабируется ровно в 2 раза (640/320); по высоте масштаб 2x даёт только
; 400 из 480 строк - остаток (80 строк, черная полоса снизу) не используется
VBE_MODE_640x480 equ 0x0101

; Константы карты памяти (❗️ MAX_ENTRIES используется и из Rust)
E820_ENTRY_SIZE  equ 24
E820_MAX_ENTRIES equ 64

; Магический заголовок файла .RLX
%define RLX_MAGIC 0xFC26

; Флаги режима архитектуры
; (16-бит: Real Mode / System API, 32-бит: Protected Mode / Ring 3)
%define RLX_MODE_16 0x16
%define RLX_MODE_32 0x32

; Системные вызовы (int 0x80)
%define SYS_PRINT_STRING 1  ; Печать строки (DS:SI для 16-бит, ESI для 32-бит)
%define SYS_PUTCHAR      2  ; Печать символа (AL для 16/32-бит)
%define SYS_EXIT         3  ; Завершение работы программы
%define SYS_READ_KEY     4  ; Ожидание нажатия клавиши (AL - символ)
%define SYS_CLEAR        5  ; Очистка экрана

%endif