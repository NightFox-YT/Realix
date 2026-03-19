; © Realix > Bootix
; (21.03.26) v0.01
; ================

; Настройка компиляции
bits 16
org 0x7C00

; Остановка CPU
halt:
    cli
    hlt
    jmp $

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55