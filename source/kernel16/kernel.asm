; © Realix > Kernel
; (05.04.26) v0.05
; ================

; Настройка компиляции
bits 16
org 0x0

; Символы
%define ENTER 0x0D, 0x0A

; > Основной код
main:
    mov si, msg_end
    call print

; > Остановка CPU
halt:
    cli
    hlt
    jmp $

; Подключение модулей
%include 'kernel16/print.asm'

; Сообщения
msg_end: db '[?] Kernel halted. This version of Realix has ended its work.', ENTER, 0