; © Realix > Config
; (13.06.26) v0.06
; ================

; Защита от повторного включения
%ifndef CONFIG_ASM
%define CONFIG_ASM

; Управляющие символы и ASCII коды
%define ENTER         0x0D, 0x0A
%define ENTER_KEY     0x0D
%define BACKSPACE_KEY 0x08
%define BEEP_CHAR     0x07

; Системные адреса памяти
PCINFO_ADDR          equ 0x4500
INITRIX_LOAD_SEGMENT equ 0x07E0
INITRIX_LOAD_OFFSET  equ 0
KERNEL_LOAD_SEGMENT  equ 0x1000
KERNEL_LOAD_OFFSET   equ 0

; Настройки VGA
VGA_WIDTH   equ 320
VGA_HEIGHT  equ 200
VGA_SEGMENT equ 0xA000

; Настройки CLI
INPUT_BUFFER_LEN equ 64

; Настройки Vault (хранилище, защищённое паролем)
VAULT_PLAIN_MAX equ 512    ; Макс. размер открытого текста (байт)
VAULT_BLOB_MAX  equ 1024   ; Макс. размер VAULT.BIN в памяти (байт)

%endif