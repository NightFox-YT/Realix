; © Realix > Bootix
; (21.03.26) v0.02
; ================

; Настройка компиляции
bits 16
org 0x7C00

; Символы
%define ENTER 0x0D, 0x0A

; Запуск
start:
    ; Настройка сегментных регистров (Напрямую настроить нельзя)
    xor ax, ax
    mov ds, ax
    mov es, ax

    ; Настройка стека (Стек растёт вниз от адреса загрузки)
    mov ss, ax
    mov sp, 0x7C00

; Основной код
main:
    mov si, msg_welcome ; "Приветствие"
    call print

; Остановка CPU
halt:
    cli
    hlt
    jmp $

; Подключение модулей
%include "kernel/print.asm"

; Сообщения
msg_welcome: db "Welcome, Realix v0.02.", ENTER, 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55