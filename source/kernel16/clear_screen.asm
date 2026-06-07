; © Realix > Clear Screen
; (07.06.26) v0.05
; ================

; > Функция очистки экрана в текстовом режиме
clear_screen:
    push ax

    mov ah, 0
    mov al, 3
    int 10h

    pop ax
    ret