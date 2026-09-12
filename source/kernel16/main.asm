; © Realix > Kernel16: Main
; (16.08.26) v0.12
; ================
; ❗️ Загружается Switcher по адресу KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

; Настройка DiskAPI
%define DISK_INIT bios_disk_init
%define DISK_READ bios_disk_read


; > Установка ядра
; Параметры:
;  - dl: номер загрузочного диска (Передаётся из Switcher - stage 2)
setup:
    ; Вывод сообщения о запуске ядра
    mov si, msg_start_kernel
    call print

    ; Инициализация драйверов (`DISK_INIT` запоминает переданный номер диска)
    call DISK_INIT
    jc disk_init_error
    call fat12_init

    ; Инициализация IVT обработчиков
    call install_exception_handlers
    call syscall16_init

    ; Получение начального значения тиков после загрузки
    call save_ticks_value

    ; "Нажмите, чтобы продолжить" с ожиданием нажатия
    mov si, msg_enter_os
    call print
    call wait_key


; > Основной код
main:
    ; Вывод заголовка Shell с очисткой экрана
    call clear_screen
    mov si, shell_title
    call print

    ; Запуск Shell (CLI - его модуль)
    call cli_run


; > Экстренная остановка выполнения кода
halt:
    cli
    hlt
    jmp $


; > Ошибка 7
disk_init_error:
    mov si, err_disk_init
    jmp error_handler


; > Обработчик ошибок
; TODO: Сделать его "мягче", чтобы не любая ошибка влекла перезагрузку
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    ; Вывод сообщения и ожидание нажатия
    call print
    call wait_key

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0


; Подключение модулей
%include 'bios-api/disk/read.asm'
%include 'bios-api/io/print.asm'
%include 'bios-api/io/print_reg.asm'
%include 'bios-api/io/cursor.asm'
%include 'bios-api/io/screen.asm'
%include 'bios-api/memory/high.asm'
%include 'bios-api/memory/low.asm'
%include 'bios-api/keyboard.asm'
%include 'bios-api/rtc.asm'
%include 'bios-api/graphics.asm'
%include 'display/memory.asm'
%include 'filesystem/fat12/file_load.asm'
%include 'kernel16/shell/cli.asm'
%include 'kernel16/shell/history.asm'
%include 'kernel16/shell/commands.asm'
%include 'kernel16/panic.asm'
%include 'kernel16/syscall.asm'

; Сообщения и строки
msg_start_kernel: db '[+] Starting kernel16.', ENTER, 0
msg_enter_os:     db ENTER, 'Press any key to continue.', 0
shell_title: 
    db 'Welcome to Realix (Real Mode with NASM kernel)...', ENTER,
    db "Type 'help' for list of commands.", ENTER, ENTER, 0

; Сообщения об ошибках
err_disk_init: db '[!] E7: Disk init failed!', ENTER, 0
