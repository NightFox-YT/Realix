; © Realix > Kernel16: Main
; (13.06.26) v0.06
; ================

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

; > Установка ядра
kernel_start:
    ; Сохраняем номер диска, переданного из switcher
    mov [boot_drive_num], dl

    ; Инициализация драйвера диска
    call disk_init
    jc disk_init_error
    call fat12_init

    mov si, msg_start_kernel
    call print

    call print_new_line

    mov si, msg_enter_os
    call print

    ; Ожидание нажатия
    mov ah, 0
    int 0x16

    ; Настройка консоли
    call cmd_cls
    mov si, cli_title
    call print
    mov si, cli_hint
    call print

main:
    call run_cli

.halt:
    cli
    hlt
    jmp $

; > Ошибки инициализации
disk_init_error:
    mov si, msg_err_disk_init
    jmp error_handler

; > Обработчик ошибок
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
%include 'kernel16/io/print_ctrl.asm'
%include 'kernel16/io/print_reg.asm'
%include 'kernel16/shell/cli.asm'
%include 'kernel16/shell/commands.asm'
%include 'bios-api/memory/high.asm'
%include 'bios-api/memory/low.asm'
%include 'bios-api/disk/read.asm'
%include 'bios-api/fat12/file_load.asm'

; Сообщения и строки
msg_start_kernel: db '[+] Starting kernel.', ENTER, 0
msg_enter_os:     db 'Press any key to continue.', 0
cli_title:        db 'Welcome to Realix (Real Mode with NASM kernel)...', ENTER, 0
cli_hint:         db "Type 'help' for list of commands.", ENTER, ENTER, 0

; Сообщения об ошибках
msg_err_disk_init: db '[!] Disk init failed!', ENTER, 0

; Переменные устройства загрузки
boot_drive_num: db 0