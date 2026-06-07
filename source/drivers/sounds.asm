; © Realix > Sounds driver
; (05.06.26) v0.05
; ================

; > Выводит на экран невидимый символ "Beep"
; ❗️ Для вызова должен быть включён текстовый режим
txt_beep:
    push ax
    push bx

    ; Вывод с TTY mode и настройками страницы и цвета
    mov ah, 0x0E
    xor bx, bx
    mov al, 0x07
    int 0x10

    pop bx
    pop ax
    ret