; © Realix > Cmd "Clear"
; (13.06.26) v0.06
; ================

; > Функция очистки экрана (Текстовый режим)
cmd_cls:
    push ax

    mov ax, 0x0003
    int 10h

    pop ax
    ret