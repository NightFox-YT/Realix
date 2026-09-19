; © Realix > IO: Cursor
; (15.08.26) v0.12
; ================


; > Выключить курсор (Текстовый режим)
cursor_set_off:
    push ax
    push cx

    mov ah, 01h
    mov cx, 2607h
    int 0x10

    pop cx
    pop ax
    ret


; > Включить курсор (Текстовый режим)
cursor_set_on:
    push ax
    push cx

    mov ah, 01h
    mov cx, 0607h
    int 0x10

    pop cx
    pop ax
    ret


; > Получение позиции курсора (Текстовый режим)
; Вывод:
;  - dh: строка позиции
;  - dl: столбец позиции
get_cursor_pos:
    push ax
    push bx

    mov ah, 03h
    xor bx, bx
    int 0x10

    pop bx
    pop ax
    ret


; > Поставить курсор на опр. позициию (Текстовый режим)
; Параметры:
;  - dh: строка позиции
;  - dl: столбец позиции
set_cursor_pos:
    push ax
    push bx

    mov ah, 02h
    xor bx, bx
    int 0x10

    pop bx
    pop ax
    ret
