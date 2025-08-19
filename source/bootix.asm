; Realix > Bootix
; (C) v0.03 | 19.08.25
; ===============

; Настройка компиляции
org 0x7C00
bits 16

; [Символы]
%define ENTER 0x0D, 0x0A

; [FAT12] Настройка (48 байт)
jmp short start
nop
%include 'fat_headers.asm'

; Запуск
start:
    ; Настройка сегментных регистров (Напрямую настроить нельзя)
    xor ax, ax
    mov ds, ax
    mov es, ax

    ; Настройка стека (Стек растёт вниз от адреса загрузки)
    mov ss, ax
    mov sp, 0x7C00

    ; Обновление номера диска (BIOS устанавливает его в dl)
    mov [ebr_drive_number], dl 

    jmp main

; Основной код
main:
    ; Чтение информации с диска
    mov ax, 1      ; LBA
    mov cl, 3      ; Кол-во секторов для чтения
    mov bx, 0x7E00 ; Адрес данных для записи (после загрузчика)
    call disk_read

    mov si, msg_welcome ; "Приветствие"
    call print

; Остановка CPU
.halt:
    cli
    hlt

; Обработчик ошибок
error_handler:
    mov ah, 0    ; Режим ожидания нажатия
    int 0x16
    jmp 0FFFFh:0 ; Переход в начало BIOS для перезагрузки

; [Kernel] Print, print_reg
%include 'print.asm'
%include 'print_reg.asm'

; [Disk] Read, lba_to_chs
%include 'read.asm'
%include 'lba_to_chs.asm'

; [Сообщения]
msg_welcome:      db 'Welcome, Realix v0.03.', ENTER, 0
msg_read_success: db '[LOG] Read ', 0, " sectors, LBA: ", 0, ENTER, 0

err_read_failed:  db '[!] Read from disk failed!', ENTER, 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55