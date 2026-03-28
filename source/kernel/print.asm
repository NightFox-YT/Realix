; © Realix > Print
; (28.03.26) v0.03
; ================

; > Вывод строки на экран
; Параметры:
;  - ds:si: адрес строки
print:
    push si
    push ax
    push bx

    mov ah, 0x0E ; TTY mode (Вывод с прокруткой курсора)
    xor bx, bx   ; Сброс номера страницы и цвета

.next_char:
    lodsb       ; Загрузка символа (si > al)
    test al, al ; Проверка на 0 (Конец строки)
    jz .done

    int 0x10
    jmp .next_char

.done:
    pop bx
    pop ax
    pop si
    ret