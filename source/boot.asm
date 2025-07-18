; Realix Bootloader (MBR)
; (C) v0.01 | 18.07.2025
; ================

; [NASM] Компиляция
org 0x7C00 ; Адрес загрузки MBR
bits 16    ; 16-битный режим

; [Entry Point]
start:
    ; Вывод символа
    mov al, "Q"  ; Символ
    mov ah, 0x0E ; 'TTY mode' (Вывод с прокруткой курсора)
    mov bh, 0    ; Номер страницы
    int 0x10     ; Вызов BIOS

; [Stop] 
.halt:
    cli       ; Запрет прерываний BIOS
    hlt       ; Остановка CPU
    jmp .halt ; Бесконечный цикл

; [Signature] 
times 510-($-$$) db 0 ; Заполнение оставшихся байтов нулём
dw 0xAA55             ; Подпись AA55 (BIOS)