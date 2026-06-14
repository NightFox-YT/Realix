; © Realix > Print
; (13.06.26) v0.06
; ================

; > Вывод строки на экран (Текстовый режим)
; Параметры:
;  - ds:si: адрес строки
print:
    push si
    push ax
    push bx

    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    xor bx, bx    ; Сброс номера страницы и цвета

.next_char:
    lodsb         ; Загрузка символа (si > al)
    test al, al   ; Достигнут ли конец строки?
    jz .done

    int 0x10
    jmp .next_char

.done:
    pop bx
    pop ax
    pop si
    ret