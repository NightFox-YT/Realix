; © Realix > Config
; (27.07.26) v0.1
; ================

; Защита от повторного включения
%ifndef CONFIG_ASM
%define CONFIG_ASM

; ! Не забывайте менять эту строку
%define OS_VERSION 'v0.11'

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

; Раскладка структуры PCINFO
PCINFO_LOW_MEM equ 0  ; Размер "нижней" памяти (КБ, word)
PCINFO_DRIVE   equ 2  ; Номер загрузочного диска (byte)
PCINFO_ENTRIES equ 3  ; Кол-во записей карты памяти (word)
PCINFO_MAP     equ 5  ; Массив записей E820

; Маркеры кластеров FAT12 (общие для bootix и bios-api/fat12)
CHAIN_END   equ 0x0FF8  ; Кластер >= этого - конец цепочки (EOF)
BAD_CLUSTER equ 0x0FF7  ; Дефектный кластер

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