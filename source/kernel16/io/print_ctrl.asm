; © Realix > Print CTRL chars
; (30.06.26) v0.07
; ================

; > Вывод символов "\n\r" на экран (Текстовый режим)
print_new_line:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символов "\n\r"
    mov al, 0x0D
    int 0x10
    mov al, 0x0A
    int 0x10

    pop bx
    pop ax
    ret

; > Вывод невидимого символа "BEL" на экран (Текстовый режим)
print_beep_char:
    push ax
    push bx

    ; Настройка TTY mode, номера страницы и цвета
    mov ah, 0x0E
    xor bx, bx

    ; Вывод символа "BEL"
    mov al, 0x07
    int 0x10

    pop bx
    pop ax
    ret