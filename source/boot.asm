; Realix Bootloader (MBR)
; (C) v0.02 | 19.07.2025
; ================

; [NASM Directives]
org 0x7C00 ; Адрес загрузки MBR
bits 16    ; 16-битный режим

; [Symbols]
%define ENTER 0x0D, 0x0A

; [Entry Point]
start:
    ; Очистка регистров. (Стек растёт вниз)
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    call clear_screen ; Очистка экрана
    jmp main

; [Kernel] Print, Clear
%include "clear.asm"
%include "print.asm"

; [Main]
main:
    mov si, msg_welcome ; "Приветствие"
    call print

; [Stop] 
.halt:
    cli       ; Запрет прерываний BIOS
    hlt       ; Остановка CPU
    jmp .halt ; Бесконечный цикл

; [Messages]
msg_welcome: db "Welcome, Realix v0.02.", ENTER, 0

; [Signature] 
times 510-($-$$) db 0 ; Заполнение оставшихся байтов нулём
dw 0xAA55             ; Подпись AA55 (BIOS)