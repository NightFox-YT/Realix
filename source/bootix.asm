; © Realix > Bootix
; (01.02.26) v0.01
; =================

; Настройка компиляции
org 0x7C00
bits 16

; Остановка CPU
halt:
    cli
    hlt

; Сигнатура AA55 (BIOS)
times 510-($-$$) db 0
dw 0xAA55