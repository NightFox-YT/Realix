; © Realix > IO: Print
; (27.07.26) v0.1
; ================
; ❗️ В режиме PRINT_MINIMAL (Bootix) доступна только `print`

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


; > Блок "Расширенный вывод" (Не для Bootix)
%ifndef PRINT_MINIMAL

; > Вывод символа из al на экран (Текстовый режим)
; Параметры:
;  - al: символ
print_char:
    push ax
    push bx

    mov ah, 0x0E  ; TTY mode (Вывод с прокруткой курсора)
    xor bx, bx    ; Сброс номера страницы и цвета
    int 0x10

    pop bx
    pop ax
    ret

%endif
