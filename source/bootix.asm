; Realix > Bootix
; (C) v0.02 | 17.08.25
; ===============

; Настройка компиляции
org 0x7C00
bits 16

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

    jmp main

; Основной код
main:
    mov si, msg_welcome ; "Приветствие"
    call print

; Остановка CPU
.halt:
    cli
    hlt

; [Kernel] Print
%include "print.asm"

; Сообщения
msg_welcome: db "Welcome, Realix v0.02.", ENTER, 0

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55