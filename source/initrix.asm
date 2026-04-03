; © Realix > Initrix
; (03.04.26) v0.04
; ================

; Настройка компиляции
bits 16
org 0x0

; Символы
%define ENTER 0x0D, 0x0A

; Основной код
main:
    ; "Инициализация..."
    mov si, msg_init
    call print

    ; "Приветствие"
    mov si, msg_welcome
    call print

; Остановка CPU
.halt:
    cli
    hlt
    jmp $

; Подключение модулей
%include 'kernel/print.asm'

; Сообщения
msg_init:    db '[+] Initializing...', ENTER, 0
msg_welcome: db 'Welcome, Realix v0.04.', ENTER, 0

; Вспомогательные строки
new_line: db ENTER, 0