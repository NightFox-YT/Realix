; Realix > Print register
; (C) v0.03 | 19.08.25
; =======================

; [Print] Вывод значения регистра
; Параметры:
;  - ax: значение нужного регистра
print_reg:
    pusha
    mov bx, 10 ; Делитель (Для перевода в десятичное число)
    xor cx, cx ; Обнуление счётчика цифр

.next_digit:
    xor dx, dx
    div bx          ; Делим ax на 10 (Получаем результат в ax, остаток в dx)
    add dl, 0x30    ; Преобразуем остаток в ASCII символ
    push dx         ; *Сохраняем цифру в стеке
    inc cx
    test ax, ax     ; Проверяем, осталось ли что-то в ax
    jnz .next_digit

.print_char:
    pop ax           ; *Достаём цифру из стека
    mov ah, 0x0E     ; TTY mode (Вывод с прокруткой курсора)
    int 0x10
    loop .print_char ; Повторяем для всех цифр

    popa
    ret