; © Realix > Initrix
; (28.03.26) v0.04
; ================

; Настройка компиляции
bits 16
org 0x0

; Символы
%define ENTER 0x0D, 0x0A

; Основной код
main:
    mov si, msg_initialization ; "Инициализация..."
    call print

    mov si, msg_welcome ; "Приветствие"
    call print

; Остановка CPU
.halt:
    cli
    hlt

; Подключение модулей
%include 'kernel/print.asm'

; Сообщения
new_line: db ENTER, 0
msg_initialization: db '[+] Starting initialization...', ENTER, 0
msg_welcome: db 'Welcome, Realix v0.04...', ENTER, 0