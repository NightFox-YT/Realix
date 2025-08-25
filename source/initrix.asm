; Realix > Initrix
; (C) v0.04 | 25.08.25
; ================

; Настройка компиляции
org 0x0
bits 16

; Символы
%define ENTER 0x0D, 0x0A

; Основной код
main:
    mov si, new_line ; "ENTER"
    call print

    mov si, msg_initialization ; "Инициализация..."
    call print

    mov si, msg_welcome ; "Приветствие"
    call print

; Остановка CPU
.halt:
    cli
    hlt

; [Kernel] Print
%include 'print.asm'

; Сообщения
new_line: db ENTER, 0
msg_initialization: db '[LOG] Starting initialization...', ENTER, 0
msg_welcome: db 'Welcome, Realix v0.03...', ENTER, 0