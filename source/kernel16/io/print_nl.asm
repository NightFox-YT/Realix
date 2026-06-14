; © Realix > Print «\n\r»
; (14.06.26) v0.06
; ================

; > Вывол символа «перевод строки» на экран (Текстовый режим)
print_new_line:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символа «\n\r»
    mov al, 0x0D
    int 0x10
    mov al, 0x0A
    int 0x10

    pop bx
    pop ax
    ret