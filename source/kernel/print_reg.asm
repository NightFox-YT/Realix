; © Realix > Print register
; (21.03.26) v0.03
; ================

; > Вывод десятичного значения регистра н экран
; Параметры:
;  - ax: значение регистра
print_reg:
    pusha
    mov bx, 10 ; Делитель (Для перевода в dec)
    xor cx, cx ; Обнуление счётчика цифр

.next_digit:
    xor dx, dx      ; Обнуление регистра с ASCII символом
    div bx          ; Делим ax на 10 (ax - реузльтат, dx - остаток)
    add dl, 0x30    ; Цифра → ASCII
    push dx         ; *Сохраняем цифру в стеке
    inc cx
    test ax, ax     ; Проверка на значение регистра
    jnz .next_digit

.print_char:
    pop ax           ; *Достаём цифру из стека
    mov ah, 0x0E     ; TTY mode (Вывод с прокруткой курсора)
    int 0x10
    loop .print_char ; Повторяем для всех цифр

    popa
    ret