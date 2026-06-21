; © Realix > Kernel
; (22.06.26) v0.07
; ================

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'config.asm'

; > Установка ядра
kernel_start:
    mov si, msg_start_kernel
    call print

    call print_new_line

    mov si, msg_enter_os
    call print

    ; Ожидание нажатия
    mov ah, 0x0
    int 0x16

    ; Настройка консоли
    call cmd_cls

    ; Гейт безопасности: загрузка и расшифровка хранилища по паролю.
    ; Без верного пароля полезный груз криптографически недоступен.
    call vault_init
    call vault_unlock

    mov si, cli_title
    call print

main:
    call run_cli

.halt:
    cli
    hlt
    jmp $


; > Обработчик ошибок (требуется модулям диска/FAT12)
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    call print

    ; Ожидание нажатия
    mov ah, 0
    int 0x16

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0


; Подключение модулей
%include 'kernel16/io/print.asm'
%include 'kernel16/io/print_nl.asm'
%include 'kernel16/io/print_reg.asm'
%include 'kernel16/shell/cli.asm'
%include 'kernel16/shell/commands.asm'
%include 'bios-api/memory/get_free.asm'
%include 'bios-api/memory/get_lower.asm'
%include 'bios-api/memory/get_map.asm'
%include 'bios-api/disk/read.asm'
%include 'bios-api/fat12/file_open.asm'
%include 'security/vault.asm'

; Сообщения и строки
msg_start_kernel: db '[+] Starting kernel.', ENTER, 0
msg_enter_os:     db 'Welcome, press any key to continue.', 0
cli_title:        db 'Realix v0.07 / (C) NightFox developer', ENTER, ENTER, 0
