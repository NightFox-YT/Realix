; © Realix > Kernel16: Main
; (27.07.26) v0.1
; ================
; ❗️ Загружается Switcher по адресу KERNEL_LOAD_SEGMENT:0, номер диска в dl

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

; > Установка ядра
kernel_start:
    ; Инициализация драйвера диска и fat12
    ; (`disk_init` запоминает номер диска, переданный из Switcher в dl)
    call disk_init
    jc disk_init_error
    call fat12_init

    ; "Запуск ядра" && "Нажмите, чтобы продолжить"
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

; > Ошибка 6
disk_init_error:
    mov si, err_disk_init
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
msg_start_kernel: db '[+] Starting kernel16.', ENTER, 0
msg_enter_os:     db 'Press any key to continue.', 0
cli_title:        db 'Welcome to Realix (Real Mode with NASM kernel)...', ENTER, 0
cli_hint:         db "Type 'help' for list of commands.", ENTER, ENTER, 0

; Сообщения об ошибках
err_disk_init: db '[!] E6: Disk init failed!', ENTER, 0
