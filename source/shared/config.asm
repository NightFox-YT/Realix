; © Realix > Config
; (17.06.26) v0.1
; ================

; Защита от повторного включения
%ifndef CONFIG_ASM
%define CONFIG_ASM

; ! Не забывайте менять эту строку
%define OS_VERSION 'v0.1'

; Управляющие символы и ASCII коды
%define ENTER 0x0D, 0x0A

; Системные адреса памяти
PCINFO_ADDR          equ 0x4500
FAT_BUFFER_ADDR      equ 0x0500
BOOTIX_LOAD_ADDR     equ 0x7C00
INITRIX_LOAD_SEGMENT equ 0x07E0
INITRIX_LOAD_OFFSET  equ 0
KERNEL_LOAD_SEGMENT  equ 0x1000
KERNEL_LOAD_OFFSET   equ 0
KERNEL32_PHYS_ADDR   equ (KERNEL_LOAD_SEGMENT*16 + KERNEL_LOAD_OFFSET)

; Маркеры кластеров FAT12 (общие для bootix и bios-api/fat12)
CHAIN_END   equ 0x0FF8  ; Кластер >= этого - конец цепочки (EOF)
BAD_CLUSTER equ 0x0FF7  ; Дефектный кластер

%endif