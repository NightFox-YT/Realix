; © Realix > Config
; (13.06.26) v0.06
; ================

; ! Не забывайте менять эту строку
%define OS_VERSION 'v0.1'

; Защита от повторного включения
%ifndef CONFIG_ASM
%define CONFIG_ASM

; Управляющие символы и ASCII коды
%define ENTER         0x0D, 0x0A
%define ENTER_KEY     0x0D
%define BACKSPACE_KEY 0x08
%define BEEP_CHAR     0x07
%define SQUARE_CHAR   0xFE

; Скан-коды расширенных клавиш (int 0x16, al=0)
%define KEY_UP_SCAN   0x48
%define KEY_DOWN_SCAN 0x50

; Системные адреса памяти
PCINFO_ADDR          equ 0x4500
INITRIX_LOAD_SEGMENT equ 0x07E0
INITRIX_LOAD_OFFSET  equ 0
KERNEL_LOAD_SEGMENT  equ 0x1000
KERNEL_LOAD_OFFSET   equ 0
KERNEL32_PHYS_ADDR   equ (KERNEL_LOAD_SEGMENT*16 + KERNEL_LOAD_OFFSET)

; Настройки модуля Memory (! MAX_ENTRIES идёт из Rust)
E820_ENTRY_SIZE  equ 24
E820_MAX_ENTRIES equ 64

; Настройки VGA
VGA_WIDTH   equ 320
VGA_HEIGHT  equ 200
VGA_SEGMENT equ 0xA000

; Настройки CLI
INPUT_BUFFER_LEN equ 64

; Размер история команд (должен быть степенью двойки)
HISTORY_SIZE      equ 8

%endif