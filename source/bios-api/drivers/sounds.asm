; © Realix > Sounds driver
; (05.06.26) v0.05
; ================

; > Выводит на экран невидимый символ "BEL" (Текстовый режим)
print_beep_char:
    push ax
    push bx

    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    xor bx, bx    ; Сброс номера страницы и цвета
    mov al, 0x07
    int 0x10

    pop bx
    pop ax
    ret